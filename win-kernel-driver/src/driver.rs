use std::path::PathBuf;
use std::ffi::OsString;
use std::ptr::null_mut;
use std::mem::size_of;
use std::ffi::{c_void, CString};
use std::convert::TryFrom;
use std::env;
use std::fs::File;
use std::io::Write;
use std::str::FromStr;

use windows_service::{
    service::{ServiceAccess, ServiceErrorControl, ServiceInfo, ServiceStartType, ServiceType, Service, ServiceStatus, ServiceState},
    service_manager::{ServiceManager, ServiceManagerAccess}
};

use winapi::um::winnt;
use winapi::um::fileapi;
use winapi::um::handleapi;
use winapi::shared::minwindef::{DWORD, LPVOID};
use winapi::um::ioapiset;
use winapi::um::errhandlingapi;
use winapi::um::winioctl;


#[repr(u32)]
pub enum Method {
    BUFFERED = 0,
    INDIRECT = 1,
    OUTDIRECT = 2,
    NEITHER = 3
}

#[repr(u32)]
pub enum Access {
    ANY = 0,
    READ = 1,
    WRITE = 2
}

pub const fn io_control_code(device_type: u32, function: u32, method: Method, access: Access) -> u32 {
    (device_type << 16) | ((access as u32) << 14) | (function << 2) | (method as u32)
}

pub struct DriverBuilder {
    device_id: &'static str,
    device_description: &'static str,
    device_type: DWORD,
    driver_path: PathBuf,
    driver_bin: Vec<u8>
}

impl DriverBuilder {
    pub fn new() -> Self {
        DriverBuilder {
            device_id: "",
            device_description: "",
            device_type: winioctl::FILE_DEVICE_UNKNOWN,
            driver_path: PathBuf::new(),
            driver_bin: vec![]
        }
    }

    pub fn set_device_id(mut self, device_id: &'static str) -> Self {
        self.device_id = device_id;
        return self;
    }

    pub fn set_device_description(mut self, device_description: &'static str) -> Self {
        self.device_description = device_description;
        return self;
    }

    pub fn set_device_type(mut self, device_type: DWORD) -> Self {
        self.device_type = device_type;
        return self;
    }

    pub fn set_driver_path(mut self, driver_path: PathBuf) -> Self {
        self.driver_path = driver_path;
        return self;
    }

    pub fn set_driver_bin(mut self, driver_bin: Vec<u8>) -> Self {
        self.driver_bin = driver_bin;
        return self;
    }

    pub fn build(&mut self) -> Result<WinKernelDriver, String> {

        if self.device_id.len() == 0 {
            return Err("Device ID needs to be set!".to_owned());
        }

        if self.driver_bin.len() == 0 && self.driver_path.components().count() == 0 {
            return Err("Either a path to the driver file, or a binary array of the driver file, must be set".to_owned());
        }

        if self.driver_bin.len() > 0 {
            let mut dir = PathBuf::from(env::temp_dir());
            dir.push(format!("{}.sys", self.device_id));

            let mut f = File::create(&dir).unwrap();
            
            let driver_bin_buffer: &[u8] = &&self.driver_bin;
            f.write_all(driver_bin_buffer).unwrap();

            self.driver_path = dir;
        }

        let driver = WinKernelDriver {
            service_name: self.device_id,
            service_description: self.device_description,
            driver_path: PathBuf::from(self.driver_path.clone()),
            device_id: self.device_id,
            device: None
        };

        Ok(driver)
    }
}

pub struct WinKernelDriver {
    service_name: &'static str,
    service_description: &'static str,
    driver_path: PathBuf,
    device_id: &'static str,
    device: Option<winnt::HANDLE>
}

impl WinKernelDriver {

    pub fn install(&self) -> Result<(), String> {

        let manager_access = ServiceManagerAccess::all();
        let service_manager_res = ServiceManager::local_computer(None::<&str>, manager_access);
        let service_manager: ServiceManager;

        match service_manager_res {
            Ok(svcman) => { service_manager = svcman; },
            Err(err) => { return Err(format!("Unable to connec to service manager: {}", err)); }
        }

        let service_info = ServiceInfo {
            name: OsString::from(self.service_name),
            display_name: OsString::from(self.service_description),
            service_type: ServiceType::KERNEL_DRIVER,
            start_type: ServiceStartType::OnDemand,
            error_control: ServiceErrorControl::Normal,
            executable_path: self.driver_path.clone(),
            launch_arguments: vec![],
            dependencies: vec![],
            account_name: None,
            account_password: None
        };

        let service = service_manager.create_service(service_info, ServiceAccess::all());
        match service {
            Ok(svc) => {
                let r = svc.start(&[OsString::from("")]);
                match r {
                    Ok(_) => { },
                    Err(err) => { return Err(format!("Failed to start service: {:?}", err)); }
                };
            },
            Err(err) => { return Err(format!("Service error: {:?}", err)); }
        };

        Ok(())
    }
    
    pub fn uninstall(&self) -> Result<(), String> {
        let manager_access = ServiceManagerAccess::all();
        let service_manager_res = ServiceManager::local_computer(None::<&str>, manager_access);
        let service_manager: ServiceManager;

        match service_manager_res {
            Ok(manager) => { service_manager = manager; },
            Err(err) => { return Err(format!("Error getting service manager: {:?}", err)); }
        }

        let service: Service;

        let open_res = service_manager.open_service(self.service_name, ServiceAccess::all());
        match open_res {
            Ok(svc) => { service = svc; },
            Err(err) => { return Err(format!("Error opening service: {:?}", err)); }
        }

        let service_status: ServiceStatus;
        match service.query_status() {
            Ok(status) => service_status = status,
            Err(err) => { return Err(format!("Error querying service status: {:?}", err)); }
        }

        if service_status.current_state != ServiceState::Stopped {
            match service.stop() {
                Ok(_) => { },
                Err(err) => { return Err(format!("Error stopping service: {:?}", err)); }
            }
        }

        match service.delete() {
            Ok(()) => { return Ok(()); },
            Err(err) => { return Err(format!("Error deleting service: {:?}", err)); }
        }
    }
    
    pub fn open(&mut self) -> Result<(), String> {

        if self.opened() {
            return Err("Driver already opened".to_string());
        }

        let mut driver_path_t: String = r"\\.\".to_string();
        driver_path_t.push_str(self.service_name);
        let driver_path = driver_path_t.as_str();

        unsafe {
            let device: winnt::HANDLE = fileapi::CreateFileA(
                CString::new(driver_path).unwrap().as_ptr(),
                winnt::GENERIC_READ | winnt::GENERIC_WRITE,
                0,
                null_mut(),
                fileapi::OPEN_EXISTING,
                winnt::FILE_ATTRIBUTE_NORMAL,
                null_mut()
            );

            if device == handleapi::INVALID_HANDLE_VALUE {
                return Err("Error occurred getting handle on kernel driver".to_string());
            } else {
                println!("Handle created");
            }
            
            self.device = Some(device);
        }

        Ok(())
    }    

    pub fn opened(&self) -> bool {
        match self.device {
            Some(_) => { return true; }
            None => { return false; }
        }
    }
    
    pub fn close(&mut self) -> Result<(), String> {

        if !self.opened() {
            return Err("Driver not opened".to_string());
        }

        let handle = self.device.unwrap();
        unsafe {
            handleapi::CloseHandle(handle);
        }        

        Ok(())
    }

    pub fn io(&self, ioctl_code: u32, mut in_buffer: u32) -> Result<u64, String> {
        if !self.opened() {
            return Err("Driver not opened!".to_string());
        }

        let mut device = self.device.unwrap() as winnt::HANDLE;

        let mut out_buffer = [0u8; size_of::<u64>()];
        let mut in_buffer_bytes = in_buffer.to_be_bytes();
        let mut in_buffer_c_void: LPVOID = &mut in_buffer_bytes as *mut _ as LPVOID;
        let mut out_buffer_c_void: LPVOID = &mut out_buffer as *mut _ as LPVOID;

        let in_buffer_size = u32::try_from(in_buffer_bytes.len()).unwrap() as DWORD;
        let out_buffer_size = out_buffer.len() as u32;
        let mut out_buffer_written: DWORD = 0;

        unsafe {
            let res = ioapiset::DeviceIoControl(
                device,
                ioctl_code,
                &mut in_buffer as *mut _ as *mut c_void,
                in_buffer_size,
                out_buffer.as_mut_ptr() as *mut _,
                out_buffer_size,
                &mut out_buffer_written,
                null_mut()
            );

            if res != 0 {
                let o = u64::from_le_bytes(out_buffer);
                return Ok(o);
            } else {
                let last_error = unsafe { errhandlingapi::GetLastError() };
                return Err(format!("DeviceIoControl - Unable to write command {:x}. Last error code: {:x}", ioctl_code, last_error));
            }
        }
    }    
}
