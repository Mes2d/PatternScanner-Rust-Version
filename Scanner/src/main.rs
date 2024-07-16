extern crate winapi;

use std::ffi::CString;
use std::ptr::null_mut;
use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::um::psapi::{GetModuleInformation, MODULEINFO};
use winapi::um::processthreadsapi::GetCurrentProcess;

fn find_pattern(module: &str, pattern: &[u8], mask: &str) -> Option<usize> {
    let module_name = CString::new(module).unwrap();
    let hmodule = unsafe { GetModuleHandleA(module_name.as_ptr()) };
    if hmodule.is_null() {
        return None;
    }

    let mut module_info = MODULEINFO {
        lpBaseOfDll: null_mut(),
        SizeOfImage: 0,
        EntryPoint: null_mut(),
    };

    unsafe {
        GetModuleInformation(GetCurrentProcess(), hmodule, &mut module_info, std::mem::size_of::<MODULEINFO>() as u32);
    }

    let module_base = module_info.lpBaseOfDll as usize;
    let module_end = module_base + module_info.SizeOfImage as usize;

    let pattern_length = mask.len();

    for i in module_base..module_end - pattern_length {
        let mut found = true;
        for j in 0..pattern_length {
            if mask.as_bytes()[j] != b'?' && pattern[j] != unsafe { *(i as *const u8).offset(j as isize) } {
                found = false;
                break;
            }
        }
        if found {
            return Some(i);
        }
    }

    None
}

fn main() {
    let ldr_load_dll: &[u8] = &[0x4C, 0x8D, 0x05, 0x18, 0x4F, 0x08, 0x00];
    let ldr_load_dll_mask = "xxxxxxx";

    let ldrp_load_dll_internal: &[u8] = &[0x8B, 0x03, 0x89, 0x44, 0x24, 0x28];
    let ldrp_load_dll_internal_mask = "xxxxxx";

    let ldrp_load_dll: &[u8] = &[0x40, 0x55, 0x53, 0x56, 0x57, 0x41, 0x56, 0x41, 0x57, 0x48, 0x8D, 0x6C, 0x24, 0x88];
    let ldrp_load_dll_mask = "xxxxxxxxxxxxxx";

    let ldr_load_dll_addr = find_pattern("ntdll.dll", ldr_load_dll, ldr_load_dll_mask);

    let ldrp_load_dll_internal_addr = find_pattern("ntdll.dll", ldrp_load_dll_internal, ldrp_load_dll_internal_mask);

    let ldrp_load_dll_addr = find_pattern("ntdll.dll", ldrp_load_dll, ldrp_load_dll_mask);

    if let Some(addr) = ldrp_load_dll_addr {
        println!("LdrpLoadDll found at address: 0x{:x}", addr);
    } else {
        println!("LdrpLoadDll not found.");
    }

    if let Some(addr) = ldr_load_dll_addr {
        println!("LdrLoadDll found at address: 0x{:x}", addr);
    } else {
        println!("LdrLoadDll addr not found.");
    }

    if let Some(addr) = ldrp_load_dll_internal_addr {
        println!("LdrpLoadDllInternal found at address: 0x{:x}", addr);
    } else {
        println!("LdrpLoadDllInternal not found.");
    }
}
