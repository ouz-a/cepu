// SPDX-License-Identifier: GPL-2.0

//! CepuCel accelerator platform and misc-device driver.

use core::cell::Cell;

use kernel::{
    c_str,
    device::{Bound, Core, Device},
    devres::Devres,
    dma::{CoherentAllocation, Device as _, DmaMask},
    fs::File,
    io::mem::IoMem,
    ioctl::{_IOC_SIZE, _IOW},
    irq,
    irq::{Flags, IrqReturn},
    miscdevice::{MiscDevice, MiscDeviceOptions, MiscDeviceRegistration},
    new_condvar, new_mutex, of, platform,
    prelude::*,
    sync::{Arc, CondVar, Mutex, aref::ARef},
    uaccess::UserSlice,
};
#[repr(C)]
#[derive(Copy, Clone)]
struct MatMulCmd {
    opcode: u32,
    _pad: u32,
    a_addr: u64,
    b_addr: u64,
    ret_addr: u64,
    m: u32,
    k: u32,
    n: u32,
    data_type: u32,
}

fn matrix_lengths(m: u32, k: u32, n: u32) -> Result<(usize, usize, usize)> {
    if m == 0 || k == 0 || n == 0 {
        return Err(EINVAL);
    }

    let a_len = m.checked_mul(k).ok_or(EOVERFLOW)?;
    let b_len = k.checked_mul(n).ok_or(EOVERFLOW)?;
    let result_len = m.checked_mul(n).ok_or(EOVERFLOW)?;

    if a_len > MATRIX_CAPACITY as u32
        || b_len > MATRIX_CAPACITY as u32
        || result_len > MATRIX_CAPACITY as u32
    {
        return Err(EINVAL);
    }

    Ok((a_len as usize, b_len as usize, result_len as usize))
}

#[repr(C)]
#[derive(Copy, Clone)]
struct MatMulRequest {
    m: u32,
    k: u32,
    n: u32,
    _reserved: u32,
    input_a_ptr: u64,
    input_b_ptr: u64,
    result_ptr: u64,
}

unsafe impl kernel::transmute::AsBytes for MatMulRequest {}
unsafe impl kernel::transmute::FromBytes for MatMulRequest {}

unsafe impl kernel::transmute::AsBytes for MatMulCmd {}
unsafe impl kernel::transmute::FromBytes for MatMulCmd {}

// Register offsets (bytes from the base)
const REG_HEAD: u32 = 0x04;
const REG_QUEUE_BASE: u32 = 0x00;
const REG_TAIL: u32 = 0x08;
const REG_CONTROL: u32 = 0x0C;
const REG_INT_STATE: u32 = 0x14;
const REG_DOOR_BELL: u32 = 0x18;

const CEPU_CEL_IOCTL_MATMUL: u32 = _IOW::<MatMulRequest>('C' as u32, 0x03);
const REG_BUFFER_SIZE: u32 = 0x1C;
const QUEUE_CAPACITY: usize = 256;
const MATRIX_CAPACITY: usize = 512;

kernel::module_platform_driver! {
    type: CepuCel,
    name: "cepu_cel",
    authors: ["Ouz"],
    description: "CepuCel accelerator driver",
    license: "GPL",
}

kernel::of_device_table!(
    OF_TABLE,
    MODULE_OF_TABLE,
    <CepuCel as platform::Driver>::IdInfo,
    [(of::DeviceId::new(c_str!("cepu,cel")), (42))]
);

struct CepuFile {
    dev: ARef<Device>,
    state: Arc<DeviceState>,
}

#[vtable]
impl MiscDevice for CepuFile {
    type Ptr = Pin<KBox<Self>>;

    fn open(_file: &File, misc: &MiscDeviceRegistration<Self>) -> Result<Self::Ptr> {
        let dev = ARef::from(misc.device());

        let owner = unsafe {
            kernel::container_of!(
                misc as *const MiscDeviceRegistration<CepuFile>,
                CepuMisc,
                registration
            )
        };

        let state_ptr = unsafe { core::ptr::addr_of!((*owner).state) };

        let state = unsafe { (&*state_ptr).clone() };

        dev_dbg!(dev, "misc device opened\n");

        KBox::pin_init(CepuFile { dev, state }, GFP_KERNEL)
    }

    fn ioctl(me: Pin<&CepuFile>, _file: &File, cmd: u32, arg: usize) -> Result<isize> {
        match cmd {
            CEPU_CEL_IOCTL_MATMUL => {
                let request_ptr = UserPtr::from_addr(arg);
                let request_size = _IOC_SIZE(cmd);

                let mut request_reader = UserSlice::new(request_ptr, request_size).reader();

                let request = request_reader.read::<MatMulRequest>()?;

                me.state.run_matmul(
                    request.m,
                    request.k,
                    request.n,
                    request.input_a_ptr,
                    request.input_b_ptr,
                    request.result_ptr,
                )?;

                dev_dbg!(me.dev, "matmul ioctl completed\n");

                Ok(0)
            }

            _ => Err(ENOTTY),
        }
    }
}

#[pin_data]
struct Data {
    #[pin]
    completion_count: Mutex<u64>,

    #[pin]
    completion_changed: CondVar,

    #[pin]
    pub iomem: Devres<IoMem<0x20>>,
}

impl Data {
    fn completion_snapshot(&self) -> u64 {
        *self.completion_count.lock()
    }

    fn signal_completion(&self) {
        {
            let mut count = self.completion_count.lock();
            *count = count.wrapping_add(1);
        }

        self.completion_changed.notify_one();
    }

    fn wait_for_completion_after(&self, previous: u64) {
        let mut count = self.completion_count.lock();

        while *count == previous {
            self.completion_changed.wait(&mut count);
        }
    }
}

impl irq::ThreadedHandler for Data {
    fn handle_threaded(&self, dev: &Device<Bound>) -> IrqReturn {
        if let Ok(io) = self.iomem.access(dev) {
            let state = io.read8_relaxed(REG_INT_STATE as usize);
            dev_dbg!(dev, "irq: int_state = {:#x}\n", state);
            io.write8_relaxed(0u8, REG_INT_STATE as usize);
        }
        self.signal_completion();
        IrqReturn::Handled
    }
}

struct SubmissionBuffers {
    queue: CoherentAllocation<MatMulCmd>,
    a: CoherentAllocation<u32>,
    b: CoherentAllocation<u32>,
    result: CoherentAllocation<u32>,
    tail: Cell<u16>,
}

#[pin_data]
struct DeviceState {
    pdev: ARef<platform::Device>,
    irq: Arc<irq::ThreadedRegistration<Data>>,

    #[pin]
    buffers: Mutex<SubmissionBuffers>,
}

impl DeviceState {
    fn run_matmul(
        &self,
        m: u32,
        k: u32,
        n: u32,
        input_a_ptr: u64,
        input_b_ptr: u64,
        result_ptr: u64,
    ) -> Result {
        let (a_len, b_len, result_len) = matrix_lengths(m, k, n)?;

        let input_a_addr = usize::try_from(input_a_ptr).map_err(|_| EFAULT)?;
        let input_b_addr = usize::try_from(input_b_ptr).map_err(|_| EFAULT)?;
        let result_addr = usize::try_from(result_ptr).map_err(|_| EFAULT)?;

        let a_bytes = a_len.checked_mul(core::mem::size_of::<u32>()).ok_or(EOVERFLOW)?;

        let b_bytes = b_len.checked_mul(core::mem::size_of::<u32>()).ok_or(EOVERFLOW)?;

        let result_bytes = result_len.checked_mul(core::mem::size_of::<u32>()).ok_or(EOVERFLOW)?;

        let mut input_a_reader = UserSlice::new(UserPtr::from_addr(input_a_addr), a_bytes).reader();

        let mut input_b_reader = UserSlice::new(UserPtr::from_addr(input_b_addr), b_bytes).reader();

        let mut result_writer =
            UserSlice::new(UserPtr::from_addr(result_addr), result_bytes).writer();

        let dev = self.pdev.as_ref();
        let buffers = self.buffers.lock();

        for index in 0..a_len {
            let value = input_a_reader.read::<u32>()?;
            kernel::dma_write!(buffers.a[index] = value)?;
        }

        for index in 0..b_len {
            let value = input_b_reader.read::<u32>()?;
            kernel::dma_write!(buffers.b[index] = value)?;
        }

        for index in 0..result_len {
            kernel::dma_write!(buffers.result[index] = 0xDEAD_BEEFu32)?;
        }

        let io = self.irq.handler().iomem.try_access().ok_or(ENODEV)?;

        let cmd = MatMulCmd {
            opcode: 0,
            _pad: 0,
            a_addr: buffers.a.dma_handle() as u64,
            b_addr: buffers.b.dma_handle() as u64,
            ret_addr: buffers.result.dma_handle() as u64,
            m,
            k,
            n,
            data_type: 2,
        };

        let queue_index = usize::from(buffers.tail.get());

        if queue_index >= QUEUE_CAPACITY {
            return Err(EIO);
        }

        let next_tail_index = (queue_index + 1) % QUEUE_CAPACITY;
        let next_tail = u16::try_from(next_tail_index).map_err(|_| EOVERFLOW)?;
        let head = io.read16_relaxed(REG_HEAD as usize);

        if next_tail == head {
            return Err(ENOSPC);
        }

        kernel::dma_write!(buffers.queue[queue_index] = cmd)?;

        buffers.tail.set(next_tail);

        io.write16_relaxed(next_tail, REG_TAIL as usize);

        let completion_before = self.irq.handler().completion_snapshot();

        dev_dbg!(dev, "ringing doorbell\n");
        io.write8_relaxed(1u8, REG_DOOR_BELL as usize);

        drop(io);

        dev_dbg!(dev, "waiting for completion\n");

        self.irq.handler().wait_for_completion_after(completion_before);

        dev_dbg!(dev, "completion signalled\n");

        for index in 0..result_len {
            let value = kernel::dma_read!(buffers.result[index])?;
            result_writer.write::<u32>(&value)?;
        }

        Ok(())
    }
}

#[pin_data]
struct CepuMisc {
    #[pin]
    registration: MiscDeviceRegistration<CepuFile>,

    state: Arc<DeviceState>,
}

struct CepuCel {
    misc: Pin<KBox<CepuMisc>>,
}

impl platform::Driver for CepuCel {
    type IdInfo = u32;
    const OF_ID_TABLE: Option<of::IdTable<Self::IdInfo>> = Some(&OF_TABLE);

    fn probe(pdev: &platform::Device<Core>, _info: Option<&u32>) -> impl PinInit<Self, Error> {
        let dev = pdev.as_ref();

        let request = pdev.io_request_by_index(0).ok_or(ENODEV)?;
        let iomem_og = request.iomap_sized::<0x20>();
        let irq_init = pdev.request_threaded_irq_by_index(
            Flags::ONESHOT,
            0,
            c_str!("cepu_cel"),
            try_pin_init!(Data {
                completion_count <- new_mutex!(0u64),
                completion_changed <- new_condvar!(),
                iomem <- iomem_og,
            }? Error),
        );

        let irq = Arc::pin_init(irq_init, GFP_KERNEL)?;
        let io = irq.handler().iomem.access(pdev.as_ref())?;

        io.write8_relaxed(0b11, REG_CONTROL as usize);
        let reg_control = io.read32_relaxed(REG_CONTROL as usize);
        dev_dbg!(dev, "control register: {}\n", reg_control);

        let mask = DmaMask::new::<32>();

        unsafe { pdev.dma_set_mask_and_coherent(mask)? };

        let queue: CoherentAllocation<MatMulCmd> =
            CoherentAllocation::alloc_coherent(dev, QUEUE_CAPACITY, GFP_KERNEL)?;
        let dma_handle = queue.dma_handle();
        dev_dbg!(dev, "queue DMA handle: {}\n", dma_handle);

        io.write32_relaxed(dma_handle as u32, REG_QUEUE_BASE as usize);
        io.write32_relaxed(QUEUE_CAPACITY as u32, REG_BUFFER_SIZE as usize);
        let q_base = io.read32_relaxed(REG_QUEUE_BASE as usize);
        dev_dbg!(dev, "queue base register: {}\n", q_base);

        let a: CoherentAllocation<u32> =
            CoherentAllocation::alloc_coherent(dev, MATRIX_CAPACITY, GFP_KERNEL)?;

        let b: CoherentAllocation<u32> =
            CoherentAllocation::alloc_coherent(dev, MATRIX_CAPACITY, GFP_KERNEL)?;

        let result: CoherentAllocation<u32> =
            CoherentAllocation::alloc_coherent(dev, MATRIX_CAPACITY, GFP_KERNEL)?;

        let state = Arc::pin_init(
            try_pin_init!(DeviceState {
                pdev: pdev.into(),
                irq,

                buffers <- new_mutex!(SubmissionBuffers {
                    queue,
                    a,
                    b,
                    result,
                    tail: Cell::new(0),
                }),
            }),
            GFP_KERNEL,
        )?;

        let misc = KBox::pin_init(
            try_pin_init!(CepuMisc {
                state,

                registration <- MiscDeviceRegistration::register(
                    MiscDeviceOptions {
                        name: c_str!("cepu-misc-device"),
                    }
                ),
            }),
            GFP_KERNEL,
        )?;

        dev_info!(misc.state.pdev.as_ref(), "initialized\n");

        Ok(Self { misc })
    }
}

impl Drop for CepuCel {
    fn drop(&mut self) {
        dev_dbg!(self.misc.state.pdev.as_ref(), "Remove Cepu cel driver.\n");
    }
}
