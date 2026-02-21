#![no_std]

//!
//! Macros that makes the assembly writing of ATT syntax more convenient.
//!
//! ## Examples
//!
//! ```rust
//! use asm_att::asm_att;
//!
//! #[cfg(target_arch = "x86_64")]
//! {
//!     fn add(left: i64, right: i64) -> i64 {
//!         let result: i64;
//!         unsafe {
//!             asm_att!(
//!                 "mov {0}, {2}",
//!                 "add {1}, {2}",
//!                 in(reg) left,
//!                 in(reg) right,
//!                 out(reg) result
//!             );
//!         }
//!         result
//!     }
//!
//!     fn str_compare(a: &str, b: &str) -> bool {
//!         if a.len() != b.len() {
//!             return false;
//!         }
//!
//!         let len = a.len();
//!         let mut result = 0_i8;
//!         unsafe {
//!             asm_att!(
//!                 "cld",
//!                 "repe cmpsb",
//!                 "sete {res}",
//!                 in("rsi") a.as_ptr(),
//!                 in("rdi") b.as_ptr(),
//!                 in("rcx") len,
//!                 res=lateout(reg_byte) result
//!             );
//!         }
//!         result != 0
//!     }
//!
//!     fn counter2(number: u64) -> u64 {
//!         let mut counter = 0;
//!         unsafe {
//!             asm_att!(
//!                 "cmp $0, {n}",
//!                 "je 2f",
//!                 "1:",
//!                 "inc {c}",
//!                 "cmp {n}, {c}",
//!                 "jne 1b",
//!                 "2:",
//!                 c=inout(reg) counter,
//!                 n=in(reg) number,
//!                 options(nostack)
//!             );
//!         }
//!         counter
//!     }
//!
//!     fn slice_copy2(src: &[u8], dst: &mut [u8]) {
//!         let (src_len, dst_len) = (src.len(), dst.len());
//!         if src_len != dst_len {
//!             panic!(
//!                 "{}: source slice length ({}) does not match destination slice length ({}).",
//!                 core::any::type_name_of_val(&slice_copy2),
//!                 src_len,
//!                 dst_len
//!             );
//!         }
//!
//!         unsafe {
//!             asm_att!(
//!                 "cld",
//!                 "movq {src}, %rsi",
//!                 "movq {dst}, %rdi",
//!                 "movq {n}, %rcx",
//!                 "rep movsb",
//!                 src=in(reg) src.as_ptr(),
//!                 dst=in(reg) dst.as_mut_ptr(),
//!                 n=in(reg) dst_len as u64,
//!                 out("rsi") _,
//!                 out("rdi") _,
//!                 out("rcx") _
//!             );
//!         }
//!     }
//!
//!     fn add_works() {
//!         let result = add(2, 3);
//!         assert_eq!(result, 5);
//!
//!         assert_eq!(add(5, 3), 8);
//!         assert_eq!(add(-1, 1), 0);
//!         assert_eq!(add(100, 200), 300);
//!     }
//!
//!     fn str_compare_works() {
//!         assert!(str_compare("", ""));
//!         assert!(str_compare("hello", "hello"));
//!         assert!(!str_compare("hello", "world"));
//!     }
//!
//!     fn counter2_works(n: u64) {
//!         for i in 0..n {
//!             assert_eq!(counter2(i), i);
//!         }
//!     }
//!
//!     fn slice_copy2_works() {
//!         const SRC: &[u8] = b"Hello World\0";
//!         const SRC_LEN: usize = SRC.len();
//!
//!         let mut dst = [0_u8; SRC_LEN];
//!         slice_copy2(&SRC, &mut dst);
//!         assert_eq!(SRC, dst);
//!
//!         {
//!             let src = str::from_utf8(SRC).unwrap();
//!             let dst = str::from_utf8(&dst).unwrap();
//!             assert_eq!(src, dst);
//!         }
//!     }
//!
//!     add_works();
//!     str_compare_works();
//!     counter2_works(200);
//!     slice_copy2_works();
//! }
//! ```

#[macro_export]
macro_rules! asm_att {
    ( $($arg:tt)+ ) => {
        ::core::arch::asm!($($arg)+, options(att_syntax));
    };
}

#[macro_export]
macro_rules! global_asm_att {
    ( $($arg:tt)+ ) => {
        ::core::arch::global_asm!($($arg)+, options(att_syntax));
    };
}

#[macro_export]
macro_rules! naked_asm_att {
    ( $($arg:tt)+ ) => {
        ::core::arch::naked_asm!($($arg)+, options(att_syntax));
    };
}

#[cfg(all(test, target_arch = "x86_64"))]
mod tests {
    use super::*;

    global_asm_att!(
        r#"
        .global add2
        add2:
            movl %edi, %eax
            addl %esi, %eax
            ret
        "#
    );

    unsafe extern "C" {
        fn add2(a: i32, b: i32) -> i32;
    }

    #[unsafe(naked)]
    #[unsafe(no_mangle)]
    unsafe extern "C" fn return_1000() -> i32 {
        naked_asm_att!("mov $1000, %eax", "ret", options(raw));
    }

    #[unsafe(naked)]
    #[unsafe(no_mangle)]
    #[cfg_attr(target_os = "macos", export_name = "\x01jmp2")]
    unsafe extern "C" fn jmp2() -> i32 {
        naked_asm_att!(
            "jmp {0}",
            "ret",
            sym return_1000
        );
    }

    #[test]
    fn add2_works() {
        assert_eq!(unsafe { add2(1, 5) }, 6);
        assert_eq!(unsafe { add2(-5, 5) }, 0);
    }

    #[test]
    fn jmp2_should_work() {
        assert_eq!(unsafe { jmp2() }, 1000);
    }
}
