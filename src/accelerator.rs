// Register offsets (bytes from base)
pub const REG_QUEUE_BASE: u32 = 0x00;
pub const REG_HEAD: u32 = 0x04;
pub const REG_TAIL: u32 = 0x08;
pub const REG_CONTROL: u32 = 0x0C;
pub const REG_STATUS: u32 = 0x10;
pub const REG_INT_STATE: u32 = 0x14;
pub const REG_DOOR_BELL: u32 = 0x18;
#[derive(Default, Debug, Clone, Copy)]
pub enum DataType {
    F8,
    F16,
    #[default]
    F32,
    F64,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct MatMul {
    pub a_addr: u64,
    pub b_addr: u64,
    pub ret_addr: u64,
    /// rows of A and result
    pub m: u32,
    /// cols of A, rows of B
    pub k: u32,
    /// cols of B and result
    pub n: u32,
    pub data_type: DataType,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct InterruptFlags {
    pub completed: bool,
    pub error: bool,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct ControlFlags {
    pub device_enabled: bool,
    pub interrupts_enabled: bool,
}

#[derive(Default, Debug, Clone, Copy)]
pub enum DeviceStatus {
    #[default]
    Idle,
    Busy,
    Error,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct CepuCel {
    pub base: u32,
    pub head: u16,
    pub tail: u16,
    pub door_bell: bool,
    pub control: ControlFlags,
    pub status: DeviceStatus,
    pub interrupt_state: InterruptFlags,
}

impl CepuCel {
    pub fn new() -> Self {
        Self::default()
    }
}

/*
1. Read
Host
Memory reads data from the CPU host memory into the Unified Buffer (UB).
_
_
2. Read
Weights reads weights from Weight Memory into the Weight FIFO as input to the Matrix Unit.
_
3. MatrixMultiply/Convolve causes the Matrix Unit to perform a matrix multiply or a convolution from the
Unified Buffer into the Accumulators. A matrix operation takes a variable-sized B*256 input, multiplies it by a
256x256 constant weight input, and produces a B*256 output, taking B pipelined cycles to complete.
4. Activate performs the nonlinear function of the artificial neuron, with options for ReLU, Sigmoid, and so on. Its
inputs are the Accumulators, and its output is the Unified Buffer. It can also perform the pooling operations needed
for convolutions using the dedicated hardware on the die, as it is connected to nonlinear function logic.
5. Write
Host
Memory writes data from the Unified Buffer into the CPU host memory.
*/

/*


> The CPU wants the device to multiply two matrices. What does the device need to know to do that job?

Memory location A ?
A: [1.1,2.2]

Memory location B ?
B: [3.3,4.4]


For matrix multiplication between A and B to work
B dimension's must match A's dimension

For example
given coordinates X and Y
Say A is a triangle at given positions
[0, 2]
[-1, -1]
[1, -1]

Say you want to rotate the spaceship 45 degrees
B = [ 0.707, -0.707 ]
    [ 0.707,  0.707 ]

M = how many things you're transforming (3 corners)
K = the dimension that connects A to B (2, because each corner has x,y and the rotation
matrix takes x,y as input)
N = what comes out per thing (2, still x,y)

- Address of A
- Address of B
- Address of result
- M, K, N
- Data type (f32)

*/


