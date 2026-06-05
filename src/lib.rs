//! Safe Rust bindings to the loupe C library for face detection and OCR.

use std::ffi::{c_int, c_void, CStr, CString};
use std::path::Path;

/// A detected face region.
#[derive(Debug, Clone)]
pub struct Face {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub confidence: f64,
}

/// A recognized text region.
#[derive(Debug, Clone)]
pub struct OcrResult {
    pub text: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub confidence: f64,
}

#[repr(C)]
struct LoupeFaceResult {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    confidence: f64,
}

#[repr(C)]
struct LoupeOcrResult {
    text: *const i8,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    confidence: f64,
}

#[cfg(not(target_os = "linux"))]
extern "C" {
    fn loupe_load_image(path: *const i8) -> *mut c_void;
    fn loupe_free_image(handle: *mut c_void);
    fn loupe_save_image(handle: *mut c_void, path: *const i8) -> c_int;
    fn loupe_detect_faces(
        handle: *mut c_void,
        out_faces: *mut *mut LoupeFaceResult,
        out_count: *mut u32,
    ) -> c_int;
    fn loupe_blur_faces(
        handle: *mut c_void,
        faces: *const LoupeFaceResult,
        count: u32,
        mode: c_int,
    ) -> *mut c_void;
    fn loupe_recognize_text(
        handle: *mut c_void,
        out_results: *mut *mut LoupeOcrResult,
        out_count: *mut u32,
    ) -> c_int;
    fn loupe_free_ocr_results(results: *mut LoupeOcrResult, count: u32);
    fn loupe_free(ptr: *mut c_void);
}

#[cfg(not(target_os = "linux"))]
pub fn detect_faces(input: &Path) -> Result<Vec<Face>, String> {
    let input_c = CString::new(input.to_str().ok_or("invalid path")?)
        .map_err(|e| format!("path: {}", e))?;

    let handle = unsafe { loupe_load_image(input_c.as_ptr()) };
    if handle.is_null() {
        return Err("loupe_load_image failed".into());
    }

    let mut faces_ptr: *mut LoupeFaceResult = std::ptr::null_mut();
    let mut count: u32 = 0;
    let status = unsafe { loupe_detect_faces(handle, &mut faces_ptr, &mut count) };
    unsafe { loupe_free_image(handle); }

    if status != 0 {
        return Err("loupe_detect_faces failed".into());
    }

    let mut faces = Vec::new();
    if count > 0 && !faces_ptr.is_null() {
        for i in 0..count as usize {
            let r = unsafe { &*faces_ptr.add(i) };
            faces.push(Face {
                x: r.x, y: r.y, width: r.width, height: r.height,
                confidence: r.confidence,
            });
        }
        unsafe { loupe_free(faces_ptr as *mut c_void); }
    }

    Ok(faces)
}

#[cfg(not(target_os = "linux"))]
pub fn blur_faces(input: &Path, output: &Path, faces: &[Face]) -> Result<(), String> {
    process_faces(input, output, faces, 0)
}

#[cfg(not(target_os = "linux"))]
pub fn redact_faces(input: &Path, output: &Path, faces: &[Face]) -> Result<(), String> {
    process_faces(input, output, faces, 1)
}

#[cfg(not(target_os = "linux"))]
fn process_faces(input: &Path, output: &Path, faces: &[Face], mode: i32) -> Result<(), String> {
    let input_c = CString::new(input.to_str().ok_or("invalid input path")?)
        .map_err(|e| format!("input: {}", e))?;
    let output_c = CString::new(output.to_str().ok_or("invalid output path")?)
        .map_err(|e| format!("output: {}", e))?;

    let handle = unsafe { loupe_load_image(input_c.as_ptr()) };
    if handle.is_null() {
        return Err("loupe_load_image failed".into());
    }

    if faces.is_empty() {
        let status = unsafe { loupe_save_image(handle, output_c.as_ptr()) };
        unsafe { loupe_free_image(handle); }
        return if status == 0 { Ok(()) } else { Err("save failed".into()) };
    }

    let c_faces: Vec<LoupeFaceResult> = faces.iter().map(|f| LoupeFaceResult {
        x: f.x, y: f.y, width: f.width, height: f.height, confidence: f.confidence,
    }).collect();

    let blurred = unsafe {
        loupe_blur_faces(handle, c_faces.as_ptr(), c_faces.len() as u32, mode)
    };
    unsafe { loupe_free_image(handle); }

    if blurred.is_null() {
        return Err("loupe_blur_faces failed".into());
    }

    let status = unsafe { loupe_save_image(blurred, output_c.as_ptr()) };
    unsafe { loupe_free_image(blurred); }

    if status != 0 { Err("save failed".into()) } else { Ok(()) }
}

#[cfg(not(target_os = "linux"))]
pub fn recognize_text(input: &Path) -> Result<Vec<OcrResult>, String> {
    let input_c = CString::new(input.to_str().ok_or("invalid path")?)
        .map_err(|e| format!("path: {}", e))?;

    let handle = unsafe { loupe_load_image(input_c.as_ptr()) };
    if handle.is_null() {
        return Err("loupe_load_image failed".into());
    }

    let mut results_ptr: *mut LoupeOcrResult = std::ptr::null_mut();
    let mut count: u32 = 0;
    let status = unsafe { loupe_recognize_text(handle, &mut results_ptr, &mut count) };
    unsafe { loupe_free_image(handle); }

    if status != 0 {
        return Err("loupe_recognize_text failed".into());
    }

    let mut results = Vec::new();
    if count > 0 && !results_ptr.is_null() {
        for i in 0..count as usize {
            let r = unsafe { &*results_ptr.add(i) };
            if !r.text.is_null() {
                let text = unsafe { CStr::from_ptr(r.text) }
                    .to_string_lossy()
                    .into_owned();
                if !text.is_empty() {
                    results.push(OcrResult {
                        text, x: r.x, y: r.y, width: r.width,
                        height: r.height, confidence: r.confidence,
                    });
                }
            }
        }
        unsafe { loupe_free_ocr_results(results_ptr, count); }
    }

    Ok(results)
}

// --- Linux stubs ---

#[cfg(target_os = "linux")]
pub fn detect_faces(_input: &Path) -> Result<Vec<Face>, String> {
    Err("face detection not available on Linux".into())
}

#[cfg(target_os = "linux")]
pub fn blur_faces(_input: &Path, _output: &Path, _faces: &[Face]) -> Result<(), String> {
    Err("face processing not available on Linux".into())
}

#[cfg(target_os = "linux")]
pub fn redact_faces(_input: &Path, _output: &Path, _faces: &[Face]) -> Result<(), String> {
    Err("face processing not available on Linux".into())
}

#[cfg(target_os = "linux")]
pub fn recognize_text(_input: &Path) -> Result<Vec<OcrResult>, String> {
    Err("OCR not available on Linux".into())
}
