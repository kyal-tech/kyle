use inkwell::attributes::{Attribute, AttributeLoc};
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::BasicType;
use inkwell::types::BasicTypeEnum;
use inkwell::values::BasicValue;
use inkwell::values::BasicValueEnum;
use inkwell::values::FunctionValue;
use inkwell::values::AnyValueEnum;
use inkwell::values::AsValueRef;
use inkwell::values::InstructionValue as InkwellInstructionValue;
use inkwell::values::PointerValue;
use inkwell::IntPredicate;
use llvm_sys::core::{LLVMSetNSW, LLVMSetNUW};
use std::collections::{HashMap, HashSet};
use kyc_mir::mir::*;
use kyc_mir::ssa::SsaValueId;

use crate::codegen::Codegen;

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn add_runtime_extern(
        &self,
        name: &str,
        ft: inkwell::types::FunctionType<'ctx>,
        kv_attrs: &[(&str, &str)],
    ) -> FunctionValue<'ctx> {
        let func = self.module.add_function(name, ft, None);
        for &(key, val) in kv_attrs {
            let attr = self.context.create_string_attribute(key, val);
            func.add_attribute(AttributeLoc::Function, attr);
        }
        func
    }

    /// Declare external runtime functions that generated code can call.
    pub(crate) fn declare_runtime_externs(&mut self) {
        let void_ty = self.context.void_type();
        let i64_ty = self.context.i64_type();
        let i32_ty = self.context.i32_type();
        let f64_ty = self.context.f64_type();
        let ptr_ty = self.context.ptr_type(Default::default());

        // void kl_print(ptr, i32)
        {
            let params = [ptr_ty.into(), i32_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_print", ft, None);
        }
        // void kl_println(ptr, i32)
        {
            let params = [ptr_ty.into(), i32_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_println", ft, None);
        }
        // ptr kl_alloc(i64)
        {
            let params = [i64_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_alloc", ft, None);
        }
        // i32 ky_free(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_free", ft, None);
        }
        // ptr kl_i64_to_str(i64)
        {
            let params = [i64_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_i64_to_str", ft, None);
        }
        // ptr kl_f64_to_str(f64)
        {
            let params = [f64_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_f64_to_str", ft, None);
        }
        // i64 kl_str_to_i64(ptr) — parse string to i64
        {
            let params = [ptr_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_str_to_i64", ft, None);
        }
        // i32 kl_str_to_i32(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_str_to_i32", ft, None);
        }
        // i32 kl_strlen(ptr) — readonly
        {
            let params = [ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.add_runtime_extern("ky_strlen", ft, &[("memory", "read")]);
        }
        // ptr kl_concat(ptr, i32, ptr, i32)
        {
            let params = [ptr_ty.into(), i32_ty.into(), ptr_ty.into(), i32_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_concat", ft, None);
        }
        // ptr kl_input()
        {
            let ft = ptr_ty.fn_type(&[], false);
            self.module.add_function("ky_input", ft, None);
        }
        // ptr kl_input_with_prompt(ptr, i32)
        {
            let params = [ptr_ty.into(), i32_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_input_with_prompt", ft, None);
        }
        // i32 kl_str_contains(ptr, ptr) — readonly
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.add_runtime_extern("ky_str_contains", ft, &[("memory", "read")]);
        }
        // i32 ky_str_index_of(ptr, ptr) — find substring index
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_str_index_of", ft, None);
        }
        // ptr ky_str_split(ptr, ptr) — split string by delimiter
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_str_split", ft, None);
        }
        // ptr kl_str_to_upper(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_str_to_upper", ft, None);
        }
        // ptr kl_str_to_lower(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_str_to_lower", ft, None);
        }
        // ptr kl_str_trim(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_str_trim", ft, None);
        }
        // ptr kl_str_replace(ptr, ptr, ptr)
        {
            let params = [ptr_ty.into(), ptr_ty.into(), ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_str_replace", ft, None);
        }
        // i32 kl_open(ptr, ptr)
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_open", ft, None);
        }
        // ptr kl_read_str(i32, i32)
        {
            let params = [i32_ty.into(), i32_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_read_str", ft, None);
        }
        // i32 kl_write_str(i32, ptr)
        {
            let params = [i32_ty.into(), ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_write_str", ft, None);
        }
        // i32 kl_close(i32)
        {
            let params = [i32_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_close", ft, None);
        }
        // ptr kl_from_cstr(ptr) — convert C string to Kyle string
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_from_cstr", ft, None);
        }
        // ptr kl_getenv(ptr) — read environment variable
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_getenv", ft, None);
        }
        // i32 kl_setenv(ptr, ptr, i32) — set environment variable
        {
            let params = [ptr_ty.into(), ptr_ty.into(), i32_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_setenv", ft, None);
        }
        // void kl_sleep(i32)
        {
            let params = [i32_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_sleep", ft, None);
        }
        // i64 kl_now()
        {
            let ft = i64_ty.fn_type(&[], false);
            self.module.add_function("ky_now", ft, None);
        }
        // i32 ky_tcp_listen(i32) — create TCP listener
        {
            let params = [i32_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_tcp_listen", ft, None);
        }
        // i32 ky_tcp_accept(i32) — accept connection
        {
            let params = [i32_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_tcp_accept", ft, None);
        }
        // ptr ky_tcp_read(i32, i32) — read from socket
        {
            let params = [i32_ty.into(), i32_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_tcp_read", ft, None);
        }
        // i32 ky_tcp_write(i32, ptr, i32) — write to socket
        {
            let params = [i32_ty.into(), ptr_ty.into(), i32_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_tcp_write", ft, None);
        }
        // i32 ky_ptr_read_i32(ptr) — read i32 from memory
        {
            let params = [ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_ptr_read_i32", ft, None);
        }
        // ptr ky_ptr_read_ptr(ptr) — read ptr from memory
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_ptr_read_ptr", ft, None);
        }
        // void ky_ptr_write_i32(ptr, i32) — write i32 to memory
        {
            let params = [ptr_ty.into(), i32_ty.into()];
            let ft = self.context.void_type().fn_type(&params, false);
            self.module.add_function("ky_ptr_write_i32", ft, None);
        }
        // void ky_ptr_write_ptr(ptr, ptr) — write pointer to memory
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = self.context.void_type().fn_type(&params, false);
            self.module.add_function("ky_ptr_write_ptr", ft, None);
        }
        // i32 ky_tcp_close(i32) — close socket
        {
            let params = [i32_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_tcp_close", ft, None);
        }
        // i32 ky_sha1(ptr, i32, ptr) — SHA-1 hash
        {
            let params = [ptr_ty.into(), i32_ty.into(), ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_sha1", ft, None);
        }
        // ptr ky_base64_encode(ptr, i32) — Base64 encode
        {
            let params = [ptr_ty.into(), i32_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_base64_encode", ft, None);
        }
        // ptr ky_ws_accept(ptr) — WebSocket handshake accept key
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_ws_accept", ft, None);
        }
        // ptr ky_ws_read_frame(i32) — read WebSocket frame
        {
            let params = [i32_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_ws_read_frame", ft, None);
        }
        // i32 ky_ws_send_frame(i32, i32, ptr, i32) — send WebSocket frame
        {
            let params = [i32_ty.into(), i32_ty.into(), ptr_ty.into(), i32_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_ws_send_frame", ft, None);
        }
        // i64 ky_pow(i64, i64)
        {
            let params = [i64_ty.into(), i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_pow", ft, None);
        }
        // i64 ky_add_pct(i64, i64)
        {
            let params = [i64_ty.into(), i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_add_pct", ft, None);
        }
        // i64 ky_sub_pct(i64, i64)
        {
            let params = [i64_ty.into(), i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_sub_pct", ft, None);
        }
        // i64 ky_mul_pct(i64, i64)
        {
            let params = [i64_ty.into(), i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_mul_pct", ft, None);
        }
        // i8 kl_char_at(ptr, i32) — readonly
        {
            let i8_ty = self.context.i8_type();
            let params = [ptr_ty.into(), i32_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.add_runtime_extern("ky_char_at", ft, &[("memory", "read")]);
        }
        // ptr ky_char_to_str(i32) — char to string
        {
            let params = [i32_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_char_to_str", ft, None);
        }
        // ptr kl_list_new()
        {
            let ft = ptr_ty.fn_type(&[], false);
            self.module.add_function("ky_list_new", ft, None);
        }
        // void kl_list_free(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_list_free", ft, None);
        }
        // ptr kl_range(i64)
        {
            let params = [i64_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_range", ft, None);
        }
        // ptr kl_range_two(i64, i64)
        {
            let params = [i64_ty.into(), i64_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_range_two", ft, None);
        }
        // void kl_list_push(ptr, i64)
        {
            let params = [ptr_ty.into(), i64_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_list_push", ft, None);
        }
        // i64 kl_list_pop(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_list_pop", ft, None);
        }
        // i64 kl_list_get(ptr, i64) — readonly
        {
            let params = [ptr_ty.into(), i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.add_runtime_extern("ky_list_get", ft, &[("memory", "read")]);
        }
        // void kl_list_set(ptr, i64, i64)
        {
            let params = [ptr_ty.into(), i64_ty.into(), i64_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_list_set", ft, None);
        }
        // i64 kl_list_len(ptr) — readonly
        {
            let params = [ptr_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.add_runtime_extern("ky_list_len", ft, &[("memory", "read")]);
        }
        // i64 kl_list_sum(ptr), kl_list_product(ptr), kl_list_max(ptr), kl_list_min(ptr) — readonly
        for name in &["ky_list_sum", "ky_list_product", "ky_list_max", "ky_list_min"] {
            let params = [ptr_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.add_runtime_extern(name, ft, &[("memory", "read")]);
        }
        // void kl_list_reverse(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_list_reverse", ft, None);
        }
        // ptr kl_list_slice(ptr, i64, i64)
        {
            let params = [ptr_ty.into(), i64_ty.into(), i64_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_list_slice", ft, None);
        }
        // void kl_list_extend(ptr, ptr)
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_list_extend", ft, None);
        }
        // i64 kl_list_pop_first(ptr) — remove and return first element
        {
            let params = [ptr_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_list_pop_first", ft, None);
        }
        // void kl_list_clear(ptr) — remove all elements
        {
            let params = [ptr_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_list_clear", ft, None);
        }
        // i32 kl_list_contains(ptr, i64) — readonly
        {
            let params = [ptr_ty.into(), i64_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.add_runtime_extern("ky_list_contains", ft, &[("memory", "read")]);
        }
        // void kl_list_insert(ptr, i64, i64) — insert at index
        {
            let params = [ptr_ty.into(), i64_ty.into(), i64_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_list_insert", ft, None);
        }
        // i64 kl_list_remove_at(ptr, i64) — remove element at index
        {
            let params = [ptr_ty.into(), i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_list_remove_at", ft, None);
        }
        // i32 ky_list_remove_value(ptr, i64) — remove first occurrence by value
        {
            let params = [ptr_ty.into(), i64_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_list_remove_value", ft, None);
        }
        // i64 ky_list_index_of(ptr, i64) — find index of element
        {
            let params = [ptr_ty.into(), i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_list_index_of", ft, None);
        }
        // void ky_list_sort(ptr) — sort list in-place
        {
            let params = [ptr_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_list_sort", ft, None);
        }
        // ptr ky_list_chunk(ptr, i64) — split list into chunks
        {
            let params = [ptr_ty.into(), i64_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_list_chunk", ft, None);
        }
        // ptr kl_list_map(ptr, ptr) — map with fn pointer
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_list_map", ft, None);
        }
        // ptr kl_list_filter(ptr, ptr) — filter with fn pointer
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_list_filter", ft, None);
        }
        // i64 kl_list_fold(ptr, i64, ptr) — fold with init + fn pointer
        {
            let params = [ptr_ty.into(), i64_ty.into(), ptr_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_list_fold", ft, None);
        }
        // i64 kl_list_reduce(ptr, ptr) — reduce with fn pointer
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_list_reduce", ft, None);
        }
        // i64 kl_iter_new(i64) — create iterator from list (i64 cast pointer)
        {
            let params = [i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_iter_new", ft, None);
        }
        // i64 kl_iter_next(i64) — get next element
        {
            let params = [i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_iter_next", ft, None);
        }
        // i64 kl_iter_map(i64, i64) — lazy map
        {
            let params = [i64_ty.into(), i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_iter_map", ft, None);
        }
        // i64 kl_iter_filter(i64, i64) — lazy filter
        {
            let params = [i64_ty.into(), i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_iter_filter", ft, None);
        }
        // ptr kl_iter_collect(i64) — collect iterator to list
        {
            let params = [i64_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_iter_collect", ft, None);
        }
        // ptr kl_str_builder_new(i64)
        {
            let params = [i64_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_str_builder_new", ft, None);
        }
        // void kl_str_builder_append(ptr, ptr)
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_str_builder_append", ft, None);
        }
        // ptr kl_str_builder_to_str(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_str_builder_to_str", ft, None);
        }
        // void kl_str_builder_free(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_str_builder_free", ft, None);
        }
        // Bootstrap str_builder declarations (without ky_ prefix)
        {
            let params = [i64_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("str_builder_new", ft, None);
        }
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("str_builder_append", ft, None);
        }
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("str_builder_to_str", ft, None);
        }
        {
            let params = [ptr_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("str_builder_free", ft, None);
        }
        // ptr kl_dict_new()
        {
            let ft = ptr_ty.fn_type(&[], false);
            self.module.add_function("ky_dict_new", ft, None);
        }
        // void kl_dict_set(ptr, ptr, i64)
        {
            let params = [ptr_ty.into(), ptr_ty.into(), i64_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_dict_set", ft, None);
        }
        // i64 kl_dict_get(ptr, ptr) — readonly
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.add_runtime_extern("ky_dict_get", ft, &[("memory", "read")]);
        }
        // i64 kl_dict_len(ptr) — readonly
        {
            let params = [ptr_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.add_runtime_extern("ky_dict_len", ft, &[("memory", "read")]);
        }
        // void kl_dict_free(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_dict_free", ft, None);
        }
        // ptr kl_substr(ptr, i64, i64)
        {
            let params = [ptr_ty.into(), i64_ty.into(), i64_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_substr", ft, None);
        }
        // i32 kl_is_digit(i32), kl_is_alpha(i32), etc. — all readnone (pure)
        {
            let params = [i32_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            for name in &["ky_is_digit", "ky_is_alpha", "ky_is_alnum",
                          "ky_is_whitespace", "ky_is_upper", "ky_is_lower",
                          "ky_ord"] {
                self.add_runtime_extern(name, ft, &[("memory", "none")]);
            }
        }
        // i32 kl_eq_str(ptr, ptr) — readonly
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.add_runtime_extern("ky_eq_str", ft, &[("memory", "read")]);
        }
        // i32 ky_str_cmp(ptr, ptr) — readonly
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.add_runtime_extern("ky_str_cmp", ft, &[("memory", "read")]);
        }
        // ptr kl_init_args(i32, ptr)  — convert C argv to Kyle list
        {
            let params = [i32_ty.into(), ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_init_args", ft, None);
        }
        // i64 kl_spawn_task(ptr, i64)  — spawn async task running extern "C" fn(i64) -> i64
        {
            let params = [ptr_ty.into(), i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_spawn_task", ft, None);
        }
        // i64 kl_await_task(i64)  — await task completion, return result
        {
            let params = [i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_await_task", ft, None);
        }
        // i32 ky_yield()  — cooperative yield hint
        {
            let ft = i32_ty.fn_type(&[], false);
            self.module.add_function("ky_yield", ft, None);
        }
        // i64 kl_spawn_thread(ptr, i64)  — spawn dedicated OS thread
        {
            let params = [ptr_ty.into(), i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_spawn_thread", ft, None);
        }
        // i64 kl_join_thread(i64)  — join thread, return result
        {
            let params = [i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_join_thread", ft, None);
        }
        // i64 kl_parallel_for(ptr, i64, i64)  — parallel for loop
        {
            let params = [ptr_ty.into(), i64_ty.into(), i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_parallel_for", ft, None);
        }
        // i64 kl_channel_new(i64)  — create channel with capacity
        {
            let params = [i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_channel_new", ft, None);
        }
        // i64 kl_channel_send(i64, i64)  — send value to channel
        {
            let params = [i64_ty.into(), i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_channel_send", ft, None);
        }
        // i64 kl_channel_recv(i64)  — receive from channel
        {
            let params = [i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_channel_recv", ft, None);
        }
        // void kl_channel_close(i64)
        {
            let params = [i64_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_channel_close", ft, None);
        }
        // i64 kl_channel_len(i64)
        {
            let params = [i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_channel_len", ft, None);
        }
        // void kl_channel_free(i64)
        {
            let params = [i64_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_channel_free", ft, None);
        }
        // i32 kl_dict_contains(ptr, ptr) — readonly
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.add_runtime_extern("ky_dict_contains", ft, &[("memory", "read")]);
        }
        // i64 kl_dict_remove(ptr, ptr)
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = self.context.i64_type().fn_type(&params, false);
            self.module.add_function("ky_dict_remove", ft, None);
        }
        // ptr ky_dict_values(ptr) — get list of values
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_dict_values", ft, None);
        }
        // ptr ky_dict_items(ptr) — get list of key-value pairs
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_dict_items", ft, None);
        }
        // ptr ky_dict_keys(ptr) — get list of keys
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_dict_keys", ft, None);
        }
        // ptr kl_set_new()
        {
            let ft = ptr_ty.fn_type(&[], false);
            self.module.add_function("ky_set_new", ft, None);
        }
        // void kl_set_free(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_set_free", ft, None);
        }
        // void kl_set_add(ptr, i64)
        {
            let params = [ptr_ty.into(), i64_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_set_add", ft, None);
        }
        // i32 kl_set_contains(ptr, i64)
        {
            let params = [ptr_ty.into(), i64_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.add_runtime_extern("ky_set_contains", ft, &[("memory", "read")]);
        }
        // i32 kl_set_remove(ptr, i64)
        {
            let params = [ptr_ty.into(), i64_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_set_remove", ft, None);
        }
        // i64 kl_set_len(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.add_runtime_extern("ky_set_len", ft, &[("memory", "read")]);
        }
        // void ky_set_clear(ptr) — remove all elements
        {
            let params = [ptr_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_set_clear", ft, None);
        }
        // ptr ky_set_union(ptr, ptr) — union of two sets
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_set_union", ft, None);
        }
        // ptr ky_set_intersection(ptr, ptr) — intersection of two sets
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_set_intersection", ft, None);
        }
        // ptr ky_set_difference(ptr, ptr) — difference of two sets
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_set_difference", ft, None);
        }
        // ptr ky_set_symmetric_difference(ptr, ptr) — symmetric difference of two sets
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_set_symmetric_difference", ft, None);
        }
        // i32 ky_set_is_subset(ptr, ptr) — check if first set is subset of second
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.add_runtime_extern("ky_set_is_subset", ft, &[("memory", "read")]);
        }
        // ptr ky_set_to_list(ptr) — convert set to list
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_set_to_list", ft, None);
        }
        // void ky_queue_clear(ptr) — remove all elements from queue
        {
            let params = [ptr_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_queue_clear", ft, None);
        }
        // void ky_stack_clear(ptr) — remove all elements from stack
        {
            let params = [ptr_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_stack_clear", ft, None);
        }
        // ptr ky_queue_to_list(ptr) — convert queue to list
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_queue_to_list", ft, None);
        }
        // ptr ky_queue_to_stack(ptr) — convert queue to stack
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_queue_to_stack", ft, None);
        }
        // ptr ky_stack_to_list(ptr) — convert stack to list
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_stack_to_list", ft, None);
        }
        // ptr ky_stack_to_queue(ptr) — convert stack to queue
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_stack_to_queue", ft, None);
        }
        // ptr ky_str_to_list(ptr) — convert string to list of chars
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_str_to_list", ft, None);
        }
        // ptr ky_deque_new()
        {
            let ft = ptr_ty.fn_type(&[], false);
            self.module.add_function("ky_deque_new", ft, None);
        }
        // void ky_deque_free(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_deque_free", ft, None);
        }
        // void ky_deque_push_back(ptr, i64)
        {
            let params = [ptr_ty.into(), i64_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_deque_push_back", ft, None);
        }
        // void ky_deque_push_front(ptr, i64)
        {
            let params = [ptr_ty.into(), i64_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_deque_push_front", ft, None);
        }
        // i64 ky_deque_pop_back(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_deque_pop_back", ft, None);
        }
        // i64 ky_deque_pop_front(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_deque_pop_front", ft, None);
        }
        // i64 ky_deque_peek_back(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_deque_peek_back", ft, None);
        }
        // i64 ky_deque_peek_front(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_deque_peek_front", ft, None);
        }
        // i64 ky_deque_len(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_deque_len", ft, None);
        }
        // void ky_deque_clear(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_deque_clear", ft, None);
        }
        // ptr ky_linked_list_new()
        {
            let ft = ptr_ty.fn_type(&[], false);
            self.module.add_function("ky_linked_list_new", ft, None);
        }
        // void ky_linked_list_free(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_linked_list_free", ft, None);
        }
        // void ky_linked_list_push_back(ptr, i64)
        {
            let params = [ptr_ty.into(), i64_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_linked_list_push_back", ft, None);
        }
        // void ky_linked_list_push_front(ptr, i64)
        {
            let params = [ptr_ty.into(), i64_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_linked_list_push_front", ft, None);
        }
        // i64 ky_linked_list_pop_back(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_linked_list_pop_back", ft, None);
        }
        // i64 ky_linked_list_pop_front(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_linked_list_pop_front", ft, None);
        }
        // i64 ky_linked_list_peek_back(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_linked_list_peek_back", ft, None);
        }
        // i64 ky_linked_list_peek_front(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_linked_list_peek_front", ft, None);
        }
        // i64 ky_linked_list_len(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_linked_list_len", ft, None);
        }
        // void ky_linked_list_clear(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_linked_list_clear", ft, None);
        }
        // ptr kl_json_parse(ptr) — parse JSON string into dict
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_json_parse", ft, None);
        }
        // ptr kl_json_stringify(ptr) — serialize dict to JSON string
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_json_stringify", ft, None);
        }
        // ptr kl_json_stringify_str(ptr) — serialize {str:str} dict to JSON
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_json_stringify_str", ft, None);
        }
        // ptr kl_clone_str(ptr) — deep-copy a heap-allocated string
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_clone_str", ft, None);
        }
        // ptr kl_clone_list(ptr) — shallow-copy a list
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_clone_list", ft, None);
        }
        // ptr kl_clone_dict(ptr) — shallow-copy a dict
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_clone_dict", ft, None);
        }
        // ptr kl_struct_to_json(ptr, ptr) — serialize struct to JSON
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_struct_to_json", ft, None);
        }
        // i32 kl_json_to_struct(ptr, ptr, ptr) — deserialize JSON to struct
        {
            let params = [ptr_ty.into(), ptr_ty.into(), ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_json_to_struct", ft, None);
        }
        // void kl_assert(i32)
        {
            let params = [i32_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_assert", ft, None);
        }
        // void kl_assert_eq(i64, i64)
        {
            let params = [i64_ty.into(), i64_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_assert_eq", ft, None);
        }
        // void kl_assert_ne(i64, i64)
        {
            let params = [i64_ty.into(), i64_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_assert_ne", ft, None);
        }
        // Declare LLVM lifetime.start/end intrinsics for stack slot reuse.
        {
            let ptr_ty = self.context.ptr_type(Default::default());
            let params = [i64_ty.into(), ptr_ty.into()];
            let lifetime_ft = void_ty.fn_type(&params, false);
            // LLVM lifetime intrinsics need mangled names for pointer types (p0 = addrspace 0)
            self.module.add_function("llvm.lifetime.start.p0", lifetime_ft, None);
            self.module.add_function("llvm.lifetime.end.p0", lifetime_ft, None);
        }

        // === Prelude types: bytes ===
        // ptr ky_bytes_new(i32)
        {
            let params = [i32_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_bytes_new", ft, None);
        }
        // i32 ky_bytes_get(ptr, i32)
        {
            let params = [ptr_ty.into(), i32_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_bytes_get", ft, None);
        }
        // void ky_bytes_set(ptr, i32, i32)
        {
            let params = [ptr_ty.into(), i32_ty.into(), i32_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_bytes_set", ft, None);
        }
        // ptr ky_bytes_to_hex(ptr, i32)
        {
            let params = [ptr_ty.into(), i32_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_bytes_to_hex", ft, None);
        }
        // void ky_bytes_free(ptr, i32)
        {
            let params = [ptr_ty.into(), i32_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_bytes_free", ft, None);
        }
        // ptr ky_bytes_from_hex(ptr, ptr)
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_bytes_from_hex", ft, None);
        }
        // ptr ky_bytes_to_base64(ptr, i32)
        {
            let params = [ptr_ty.into(), i32_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_bytes_to_base64", ft, None);
        }

        // === Prelude types: decimal ===
        // i64 ky_decimal_from_str(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_decimal_from_str", ft, None);
        }
        // ptr ky_decimal_to_str(i64)
        {
            let params = [i64_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_decimal_to_str", ft, None);
        }
        // i64 ky_decimal_round(i64, i32)
        {
            let params = [i64_ty.into(), i32_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_decimal_round", ft, None);
        }
        // i64 ky_decimal_truncate(i64)
        {
            let params = [i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_decimal_truncate", ft, None);
        }

        // === Prelude types: regex ===
        // ptr ky_regex_new(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_regex_new", ft, None);
        }
        // i32 ky_regex_is_match(ptr, ptr)
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_regex_is_match", ft, None);
        }
        // ptr ky_regex_find(ptr, ptr)
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_regex_find", ft, None);
        }
        // ptr ky_regex_replace(ptr, ptr, ptr)
        {
            let params = [ptr_ty.into(), ptr_ty.into(), ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_regex_replace", ft, None);
        }
        // void ky_regex_free(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_regex_free", ft, None);
        }

        // === Prelude types: uuid ===
        // ptr ky_uuid_v4()
        {
            let ft = ptr_ty.fn_type(&[], false);
            self.module.add_function("ky_uuid_v4", ft, None);
        }
        // ptr ky_uuid_parse(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_uuid_parse", ft, None);
        }

        // === Prelude types: url ===
        // ptr ky_url_scheme(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_url_scheme", ft, None);
        }
        // ptr ky_url_host(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_url_host", ft, None);
        }
        // i32 ky_url_port(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_url_port", ft, None);
        }
        // ptr ky_url_path(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_url_path", ft, None);
        }
        // ptr ky_url_query(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_url_query", ft, None);
        }

        // === Crypto ===
        // ptr ky_sha256(ptr, i32, ptr)
        {
            let params = [ptr_ty.into(), i32_ty.into(), ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_sha256", ft, None);
        }
        // i32 ky_random_bytes(ptr, i64)
        {
            let params = [ptr_ty.into(), i64_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_random_bytes", ft, None);
        }

        // === Prelude types: datetime ===
        // i64 ky_datetime_now()
        {
            let ft = i64_ty.fn_type(&[], false);
            self.module.add_function("ky_datetime_now", ft, None);
        }
        // i64 ky_datetime_parse(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_datetime_parse", ft, None);
        }
        // ptr ky_datetime_format(i64, ptr)
        {
            let params = [i64_ty.into(), ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_datetime_format", ft, None);
        }
        // i32 ky_datetime_year(i64)
        {
            let params = [i64_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_datetime_year", ft, None);
        }
        // i32 ky_datetime_month(i64)
        {
            let params = [i64_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_datetime_month", ft, None);
        }
        // i32 ky_datetime_day(i64)
        {
            let params = [i64_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_datetime_day", ft, None);
        }
        // i32 ky_datetime_hour(i64)
        {
            let params = [i64_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_datetime_hour", ft, None);
        }
        // i32 ky_datetime_minute(i64)
        {
            let params = [i64_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_datetime_minute", ft, None);
        }
        // i32 ky_datetime_second(i64)
        {
            let params = [i64_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_datetime_second", ft, None);
        }
        // i64 ky_datetime_add_days(i64, i32)
        {
            let params = [i64_ty.into(), i32_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_datetime_add_days", ft, None);
        }
        // i64 ky_datetime_add_hours(i64, i32)
        {
            let params = [i64_ty.into(), i32_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_datetime_add_hours", ft, None);
        }
        // i64 ky_datetime_diff(i64, i64)
        {
            let params = [i64_ty.into(), i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_datetime_diff", ft, None);
        }
        // i64 ky_datetime_from_ymdhms(i32, i32, i32, i32, i32, i32)
        {
            let params = [i32_ty.into(), i32_ty.into(), i32_ty.into(), i32_ty.into(), i32_ty.into(), i32_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_datetime_from_ymdhms", ft, None);
        }

        // === Prelude types: date ===
        // i32 ky_date_today()
        {
            let ft = i32_ty.fn_type(&[], false);
            self.module.add_function("ky_date_today", ft, None);
        }
        // i32 ky_date_from_ymd(i32, i32, i32)
        {
            let params = [i32_ty.into(), i32_ty.into(), i32_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_date_from_ymd", ft, None);
        }
        // i32 ky_date_parse(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_date_parse", ft, None);
        }
        // i32 ky_date_year(i32)
        {
            let params = [i32_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_date_year", ft, None);
        }
        // i32 ky_date_month(i32)
        {
            let params = [i32_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_date_month", ft, None);
        }
        // i32 ky_date_day(i32)
        {
            let params = [i32_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_date_day", ft, None);
        }
        // i32 ky_date_weekday(i32)
        {
            let params = [i32_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_date_weekday", ft, None);
        }
        // i32 ky_date_add_days(i32, i32)
        {
            let params = [i32_ty.into(), i32_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_date_add_days", ft, None);
        }
        // ptr ky_date_format(i32, ptr)
        {
            let params = [i32_ty.into(), ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_date_format", ft, None);
        }

        // === Prelude types: time ===
        // i32 ky_time_now()
        {
            let ft = i32_ty.fn_type(&[], false);
            self.module.add_function("ky_time_now", ft, None);
        }
        // i32 ky_time_from_hms(i32, i32, i32)
        {
            let params = [i32_ty.into(), i32_ty.into(), i32_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_time_from_hms", ft, None);
        }
        // i32 ky_time_parse(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_time_parse", ft, None);
        }
        // i32 ky_time_hour(i32)
        {
            let params = [i32_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_time_hour", ft, None);
        }
        // i32 ky_time_minute(i32)
        {
            let params = [i32_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_time_minute", ft, None);
        }
        // i32 ky_time_second(i32)
        {
            let params = [i32_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_time_second", ft, None);
        }

        // === FS / Path operations ===
        // i32 ky_fs_exists(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_fs_exists", ft, None);
        }
        // i32 ky_fs_is_dir(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_fs_is_dir", ft, None);
        }
        // i32 ky_fs_is_file(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_fs_is_file", ft, None);
        }
        // i64 ky_fs_size(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_fs_size", ft, None);
        }
        // i32 ky_fs_copy(ptr, ptr)
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_fs_copy", ft, None);
        }
        // i32 ky_fs_remove(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_fs_remove", ft, None);
        }
        // i32 ky_fs_create_dir(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_fs_create_dir", ft, None);
        }
        // i32 ky_fs_remove_dir(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_fs_remove_dir", ft, None);
        }
        // i32 ky_fs_rename(ptr, ptr)
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_fs_rename", ft, None);
        }
        // ptr ky_fs_read_to_string(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_fs_read_to_string", ft, None);
        }
        // i32 ky_fs_write_string(ptr, ptr)
        {
            let params = [ptr_ty.into(), ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_fs_write_string", ft, None);
        }
        // i64 ky_fs_list_dir(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_fs_list_dir", ft, None);
        }

        // === Duration ===
        // i64 ky_duration_from_secs(i64)
        {
            let params = [i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_duration_from_secs", ft, None);
        }
        // i64 ky_duration_from_millis(i64)
        {
            let params = [i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_duration_from_millis", ft, None);
        }
        // i64 ky_duration_from_hours(i64)
        {
            let params = [i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_duration_from_hours", ft, None);
        }
        // i64 ky_duration_from_days(i64)
        {
            let params = [i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_duration_from_days", ft, None);
        }
        // ptr ky_duration_to_str(i64)
        {
            let params = [i64_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_duration_to_str", ft, None);
        }
        // void ky_duration_free(i64)
        {
            let params = [i64_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_duration_free", ft, None);
        }

        // === Path ===
        // i64 ky_path_new(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_path_new", ft, None);
        }
        // ptr ky_path_dirname(i64)
        {
            let params = [i64_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_path_dirname", ft, None);
        }
        // ptr ky_path_basename(i64)
        {
            let params = [i64_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_path_basename", ft, None);
        }
        // ptr ky_path_extension(i64)
        {
            let params = [i64_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_path_extension", ft, None);
        }
        // i64 ky_path_join(i64, ptr)
        {
            let params = [i64_ty.into(), ptr_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_path_join", ft, None);
        }
        // ptr ky_path_to_str(i64)
        {
            let params = [i64_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_path_to_str", ft, None);
        }
        // void ky_path_free(i64)
        {
            let params = [i64_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_path_free", ft, None);
        }

        // === BigInt ===
        // i64 ky_big_int_from_str(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_big_int_from_str", ft, None);
        }
        // i64 ky_big_int_from_i64(i64)
        {
            let params = [i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_big_int_from_i64", ft, None);
        }
        // i64 ky_big_int_add(i64, i64)
        {
            let params = [i64_ty.into(), i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_big_int_add", ft, None);
        }
        // i64 ky_big_int_sub(i64, i64)
        {
            let params = [i64_ty.into(), i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_big_int_sub", ft, None);
        }
        // i64 ky_big_int_mul(i64, i64)
        {
            let params = [i64_ty.into(), i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_big_int_mul", ft, None);
        }
        // ptr ky_big_int_to_str(i64)
        {
            let params = [i64_ty.into()];
            let ft = ptr_ty.fn_type(&params, false);
            self.module.add_function("ky_big_int_to_str", ft, None);
        }
        // void ky_big_int_free(i64)
        {
            let params = [i64_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_big_int_free", ft, None);
        }

        // === RC / ARC (reference counting) ===
        // i32 ky_retain(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_retain", ft, None);
        }
        // i32 ky_release(ptr)
        {
            let params = [ptr_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_release", ft, None);
        }

        // === Mutex ===
        // i64 ky_mutex_new(i64)
        {
            let params = [i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_mutex_new", ft, None);
        }
        // i64 ky_mutex_lock(i64)
        {
            let params = [i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_mutex_lock", ft, None);
        }
        // void ky_mutex_store(i64, i64)
        {
            let params = [i64_ty.into(), i64_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_mutex_store", ft, None);
        }
        // void ky_mutex_free(i64)
        {
            let params = [i64_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_mutex_free", ft, None);
        }

        // === AtomicI64 ===
        // i64 ky_atomic_i64_new(i64)
        {
            let params = [i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_atomic_i64_new", ft, None);
        }
        // i64 ky_atomic_i64_load(i64)
        {
            let params = [i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_atomic_i64_load", ft, None);
        }
        // void ky_atomic_i64_store(i64, i64)
        {
            let params = [i64_ty.into(), i64_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_atomic_i64_store", ft, None);
        }
        // i64 ky_atomic_i64_add(i64, i64)
        {
            let params = [i64_ty.into(), i64_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_atomic_i64_add", ft, None);
        }
        // void ky_atomic_i64_free(i64)
        {
            let params = [i64_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_atomic_i64_free", ft, None);
        }

        // === AtomicBool ===
        // i64 ky_atomic_bool_new(i32)
        {
            let params = [i32_ty.into()];
            let ft = i64_ty.fn_type(&params, false);
            self.module.add_function("ky_atomic_bool_new", ft, None);
        }
        // i32 ky_atomic_bool_load(i64)
        {
            let params = [i64_ty.into()];
            let ft = i32_ty.fn_type(&params, false);
            self.module.add_function("ky_atomic_bool_load", ft, None);
        }
        // void ky_atomic_bool_store(i64, i32)
        {
            let params = [i64_ty.into(), i32_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_atomic_bool_store", ft, None);
        }
        // void ky_atomic_bool_free(i64)
        {
            let params = [i64_ty.into()];
            let ft = void_ty.fn_type(&params, false);
            self.module.add_function("ky_atomic_bool_free", ft, None);
        }
    }

    /// Emit inline LLVM IR for list operations instead of calling FFI functions.
    /// KlList layout (24 bytes): [ptr: data, i64: len, i64: cap] at offsets 0, 8, 16.
    pub(crate) fn emit_inline_list_op(
        &self,
        name: &str,
        dest: &Option<usize>,
        args: &[MirValue],
        last_value_map: &mut HashMap<usize, BasicValueEnum<'ctx>>,
    ) -> Result<(), String> {
        let i64_ty = self.context.i64_type();
        let i8_ty = self.context.i8_type();
        let ptr_ty = self.context.ptr_type(Default::default());

        let list_arg = self.value_to_llvm(&args[0], last_value_map)?;
        let list_i64 = match list_arg {
            BasicValueEnum::IntValue(iv) => iv,
            _ => return Err("inline list: expected i64 list pointer".to_string()),
        };
        let list_ptr = self.builder.build_int_to_ptr(list_i64, ptr_ty, "_lptr")
            .map_err(|e| format!("inline list inttoptr: {}", e))?;

        match name {
            "ky_list_len" => {
                let off8 = i64_ty.const_int(8, false);
                let len_ptr = unsafe {
                    self.builder.build_in_bounds_gep(i8_ty, list_ptr, &[off8], "_lenp")
                        .map_err(|e| format!("list_len gep: {}", e))?
                };
                let len_val = self.builder.build_load(i64_ty, len_ptr, "_len")
                    .map_err(|e| format!("list_len load: {}", e))?;
                self.tbaa_load(len_val, "list_len");
                if let Some(d) = dest {
                    if let Some(alloca_ptr) = self.alloca_map.get(*d).and_then(|p| *p) {
                        let sv = self.builder.build_store(alloca_ptr, len_val)
                            .map_err(|e| format!("list_len store: {}", e))?;
                        self.tbaa_store(sv, "int");
                    }
                    last_value_map.insert(*d, len_val);
                }
            }
            "ky_list_get" if args.len() >= 2 => {
                let idx_arg = self.value_to_llvm(&args[1], last_value_map)?;
                let idx_i64 = match idx_arg {
                    BasicValueEnum::IntValue(iv) => iv,
                    _ => return Err("ky_list_get: expected i64 index".to_string()),
                };
                // Load data pointer (TBAA list_data — doesn't alias with array elements)
                let zero = i64_ty.const_int(0, false);
                let data_ptr_ptr = unsafe {
                    self.builder.build_in_bounds_gep(i8_ty, list_ptr, &[zero], "_dpp")
                        .map_err(|e| format!("list_get gep data: {}", e))?
                };
                let data_load = self.builder.build_load(ptr_ty, data_ptr_ptr, "_data")
                    .map_err(|e| format!("list_get load data: {}", e))?;
                self.tbaa_load(data_load, "list_data");
                let data = data_load.into_pointer_value();
                // GEP into data array
                let elem_ptr = unsafe {
                    self.builder.build_in_bounds_gep(i64_ty, data, &[idx_i64], "_elem")
                        .map_err(|e| format!("list_get gep elem: {}", e))?
                };
                let val = self.builder.build_load(i64_ty, elem_ptr, "_val")
                    .map_err(|e| format!("list_get load val: {}", e))?;
                self.tbaa_load(val, "int");
                if let Some(d) = dest {
                    if let Some(alloca_ptr) = self.alloca_map.get(*d).and_then(|p| *p) {
                        let sv = self.builder.build_store(alloca_ptr, val)
                            .map_err(|e| format!("list_get store: {}", e))?;
                        self.tbaa_store(sv, "int");
                    }
                    last_value_map.insert(*d, val);
                }
            }
            "ky_list_pop" if args.len() >= 1 => {
                // Load length at offset 8
                let off8 = i64_ty.const_int(8, false);
                let len_ptr = unsafe {
                    self.builder.build_in_bounds_gep(i8_ty, list_ptr, &[off8], "_lenp")
                        .map_err(|e| format!("pop gep len: {}", e))?
                };
                let len_val = self.builder.build_load(i64_ty, len_ptr, "_len")
                    .map_err(|e| format!("pop load len: {}", e))?;
                self.tbaa_load(len_val, "list_len");
                let one = i64_ty.const_int(1, false);
                let new_len = self.builder.build_int_sub(len_val.into_int_value(), one, "_newlen")
                    .map_err(|e| format!("pop sub: {}", e))?;
                self.builder.build_store(len_ptr, new_len)
                    .map_err(|e| format!("pop store len: {}", e))?;
                let zero = i64_ty.const_int(0, false);
                let data_ptr_ptr = unsafe {
                    self.builder.build_in_bounds_gep(i8_ty, list_ptr, &[zero], "_dpp")
                        .map_err(|e| format!("pop gep data: {}", e))?
                };
                let data_load = self.builder.build_load(ptr_ty, data_ptr_ptr, "_data")
                    .map_err(|e| format!("pop load data: {}", e))?;
                self.tbaa_load(data_load, "list_data");
                let data = data_load.into_pointer_value();
                let elem_ptr = unsafe {
                    self.builder.build_in_bounds_gep(i64_ty, data, &[new_len], "_elem")
                        .map_err(|e| format!("pop gep elem: {}", e))?
                };
                let val = self.builder.build_load(i64_ty, elem_ptr, "_val")
                    .map_err(|e| format!("pop load val: {}", e))?;
                self.tbaa_load(val, "int");
                if let Some(d) = dest {
                    if let Some(alloca_ptr) = self.alloca_map.get(*d).and_then(|p| *p) {
                        let sv = self.builder.build_store(alloca_ptr, val)
                            .map_err(|e| format!("pop store: {}", e))?;
                        self.tbaa_store(sv, "int");
                    }
                    last_value_map.insert(*d, val);
                }
            }
            "ky_list_set" if args.len() >= 3 => {
                let idx_arg = self.value_to_llvm(&args[1], last_value_map)?;
                let val_arg = self.value_to_llvm(&args[2], last_value_map)?;
                let idx_i64 = match idx_arg {
                    BasicValueEnum::IntValue(iv) => iv,
                    _ => return Err("ky_list_set: expected i64 index".to_string()),
                };
                let val_i64 = match val_arg {
                    BasicValueEnum::IntValue(iv) => iv,
                    _ => return Err("ky_list_set: expected i64 value".to_string()),
                };
                // Load data pointer (TBAA list_data — doesn't alias with element stores)
                let zero = i64_ty.const_int(0, false);
                let data_ptr_ptr = unsafe {
                    self.builder.build_in_bounds_gep(i8_ty, list_ptr, &[zero], "_dpp")
                        .map_err(|e| format!("list_set gep data: {}", e))?
                };
                let data_load = self.builder.build_load(ptr_ty, data_ptr_ptr, "_data")
                    .map_err(|e| format!("list_set load data: {}", e))?;
                self.tbaa_load(data_load, "list_data");
                let data = data_load.into_pointer_value();
                let elem_ptr = unsafe {
                    self.builder.build_in_bounds_gep(i64_ty, data, &[idx_i64], "_elem")
                        .map_err(|e| format!("list_set gep elem: {}", e))?
                };
                let sv = self.builder.build_store(elem_ptr, val_i64)
                    .map_err(|e| format!("list_set store: {}", e))?;
                self.tbaa_store(sv, "int");
            }
            _ => return Err(format!("unknown inline list op: {}", name)),
        }
        Ok(())
    }

    /// SSA version of inline list operations. Uses block_vals directly instead of last_value_map.
    /// Reads SsaValueId from block_vals (same as ssa_read! macro).
    pub(crate) fn emit_ssa_inline_list_op(
        &self,
        name: &str,
        block_vals: &HashMap<usize, BasicValueEnum<'ctx>>,
        args: &[SsaValueId],
    ) -> Result<Option<BasicValueEnum<'ctx>>, String> {
        let i64_ty = self.context.i64_type();
        let i8_ty = self.context.i8_type();
        let ptr_ty = self.context.ptr_type(Default::default());

        let read_val = |id: SsaValueId| -> BasicValueEnum<'ctx> {
            block_vals.get(&id).copied().unwrap_or(self.context.i32_type().const_zero().as_basic_value_enum())
        };
        let list_val = read_val(args[0]);
        let list_ptr = match &list_val {
            BasicValueEnum::IntValue(iv) => self.builder.build_int_to_ptr(*iv, ptr_ty, "_lptr")
                .map_err(|e| format!("ssa inline list inttoptr: {}", e))?,
            BasicValueEnum::PointerValue(pv) => *pv,
            _ => return Err("ssa inline list: expected ptr or i64".to_string()),
        };

        match name {
            "ky_list_len" => {
                let off8 = i64_ty.const_int(8, false);
                let len_ptr = unsafe {
                    self.builder.build_in_bounds_gep(i8_ty, list_ptr, &[off8], "_lenp")
                        .map_err(|e| format!("ssa list_len gep: {}", e))?
                };
                let len_val = self.builder.build_load(i64_ty, len_ptr, "_len")
                    .map_err(|e| format!("ssa list_len load: {}", e))?;
                self.tbaa_load(len_val, "list_len");
                Ok(Some(len_val))
            }
            "ky_list_get" if args.len() >= 2 => {
                let idx_val = read_val(args[1]);
                let idx_i64 = match idx_val {
                    BasicValueEnum::IntValue(iv) => iv,
                    _ => return Err("ky_list_get: expected i64 index".to_string()),
                };
                let zero = i64_ty.const_int(0, false);
                let data_ptr_ptr = unsafe {
                    self.builder.build_in_bounds_gep(i8_ty, list_ptr, &[zero], "_dpp")
                        .map_err(|e| format!("ssa list_get gep data: {}", e))?
                };
                let data_load = self.builder.build_load(ptr_ty, data_ptr_ptr, "_data")
                    .map_err(|e| format!("ssa list_get load data: {}", e))?;
                self.tbaa_load(data_load, "list_data");
                let data = data_load.into_pointer_value();
                let elem_ptr = unsafe {
                    self.builder.build_in_bounds_gep(i64_ty, data, &[idx_i64], "_elem")
                        .map_err(|e| format!("ssa list_get gep elem: {}", e))?
                };
                let val = self.builder.build_load(i64_ty, elem_ptr, "_val")
                    .map_err(|e| format!("ssa list_get load val: {}", e))?;
                self.tbaa_load(val, "int");
                Ok(Some(val))
            }
            "ky_list_pop" if args.len() >= 1 => {
                // Load length at offset 8 in KlList struct
                let off8 = i64_ty.const_int(8, false);
                let len_ptr = unsafe {
                    self.builder.build_in_bounds_gep(i8_ty, list_ptr, &[off8], "_lenp")
                        .map_err(|e| format!("ssa list_pop gep len: {}", e))?
                };
                let len_val = self.builder.build_load(i64_ty, len_ptr, "_len")
                    .map_err(|e| format!("ssa list_pop load len: {}", e))?;
                self.tbaa_load(len_val, "list_len");
                // Decrement length
                let one = i64_ty.const_int(1, false);
                let new_len = self.builder.build_int_sub(len_val.into_int_value(), one, "_newlen")
                    .map_err(|e| format!("ssa list_pop sub: {}", e))?;
                self.builder.build_store(len_ptr, new_len)
                    .map_err(|e| format!("ssa list_pop store len: {}", e))?;
                // Read the element at the old last position (new_len)
                let zero = i64_ty.const_int(0, false);
                let data_ptr_ptr = unsafe {
                    self.builder.build_in_bounds_gep(i8_ty, list_ptr, &[zero], "_dpp")
                        .map_err(|e| format!("ssa list_pop gep data: {}", e))?
                };
                let data_load = self.builder.build_load(ptr_ty, data_ptr_ptr, "_data")
                    .map_err(|e| format!("ssa list_pop load data: {}", e))?;
                self.tbaa_load(data_load, "list_data");
                let data = data_load.into_pointer_value();
                let elem_ptr = unsafe {
                    self.builder.build_in_bounds_gep(i64_ty, data, &[new_len], "_elem")
                        .map_err(|e| format!("ssa list_pop gep elem: {}", e))?
                };
                let val = self.builder.build_load(i64_ty, elem_ptr, "_val")
                    .map_err(|e| format!("ssa list_pop load val: {}", e))?;
                self.tbaa_load(val, "int");
                Ok(Some(val))
            }
            "ky_list_set" if args.len() >= 3 => {
                let idx_val = read_val(args[1]);
                let val_val = read_val(args[2]);
                let idx_i64 = match idx_val {
                    BasicValueEnum::IntValue(iv) => iv,
                    _ => return Err("ky_list_set: expected i64 index".to_string()),
                };
                let val_i64 = match val_val {
                    BasicValueEnum::IntValue(iv) => iv,
                    _ => return Err("ky_list_set: expected i64 value".to_string()),
                };
                let zero = i64_ty.const_int(0, false);
                let data_ptr_ptr = unsafe {
                    self.builder.build_in_bounds_gep(i8_ty, list_ptr, &[zero], "_dpp")
                        .map_err(|e| format!("ssa list_set gep data: {}", e))?
                };
                let data_load = self.builder.build_load(ptr_ty, data_ptr_ptr, "_data")
                    .map_err(|e| format!("ssa list_set load data: {}", e))?;
                self.tbaa_load(data_load, "list_data");
                let data = data_load.into_pointer_value();
                let elem_ptr = unsafe {
                    self.builder.build_in_bounds_gep(i64_ty, data, &[idx_i64], "_elem")
                        .map_err(|e| format!("ssa list_set gep elem: {}", e))?
                };
                let sv = self.builder.build_store(elem_ptr, val_i64)
                    .map_err(|e| format!("ssa list_set store: {}", e))?;
                self.tbaa_store(sv, "int");
                Ok(None)
            }
            _ => Err(format!("unknown ssa inline list op: {}", name)),
        }
    }

}
