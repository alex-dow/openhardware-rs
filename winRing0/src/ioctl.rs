use winKernelDriver::io_control_code;
use winKernelDriver::Method;
use winKernelDriver::Access;

pub const DEVICE_TYPE: u32 = 40000;

#[repr(u32)]
pub enum IOCTL {
    OLS_GET_DRIVER_VERSION = io_control_code(DEVICE_TYPE, 0x800, Method::BUFFERED, Access::ANY),
    OLS_GET_REFCOUNT = io_control_code(DEVICE_TYPE, 0x801, Method::BUFFERED, Access::ANY),
    OLS_READ_MSR = io_control_code(DEVICE_TYPE, 0x821, Method::BUFFERED, Access::ANY),
    OLS_WRITE_MSR = io_control_code(DEVICE_TYPE, 0x822, Method::BUFFERED, Access::ANY),
    OLS_READ_PMC = io_control_code(DEVICE_TYPE, 0x823, Method::BUFFERED, Access::ANY),
    OLS_HALT = io_control_code(DEVICE_TYPE, 0x824, Method::BUFFERED, Access::ANY),
    OLS_READ_IO_PORT =  io_control_code(DEVICE_TYPE, 0x831, Method::BUFFERED, Access::READ),
    OLS_WRITE_IO_PORT = io_control_code(DEVICE_TYPE, 0x832, Method::BUFFERED, Access::WRITE),
    OLS_READ_IO_PORT_BYTE = io_control_code(DEVICE_TYPE, 0x833, Method::BUFFERED, Access::READ),
    OLS_READ_IO_PORT_WORD = io_control_code(DEVICE_TYPE, 0x834, Method::BUFFERED, Access::READ),
    OLS_READ_IO_PORT_DWORD = io_control_code(DEVICE_TYPE, 0x835, Method::BUFFERED, Access::READ),
    OLS_WRITE_IO_PORT_BYTE = io_control_code(DEVICE_TYPE, 0x836, Method::BUFFERED, Access::WRITE),
    OLS_WRITE_IO_PORT_WORD = io_control_code(DEVICE_TYPE, 0x837, Method::BUFFERED, Access::WRITE),
    OLS_WRITE_IO_PORT_DWORD = io_control_code(DEVICE_TYPE, 0x838, Method::BUFFERED, Access::WRITE),
    OLD_READ_MEMORY = io_control_code(DEVICE_TYPE, 0x841, Method::BUFFERED, Access::READ),
    OLS_WRITE_MEMORY = io_control_code(DEVICE_TYPE, 0x842, Method::BUFFERED, Access::WRITE),
    OLS_READ_PCI_CONFIG = io_control_code(DEVICE_TYPE, 0x851, Method::BUFFERED, Access::READ),
    OLS_WRITE_PCI_CONFIG = io_control_code(DEVICE_TYPE, 0x852, Method::BUFFERED, Access::READ)
}

