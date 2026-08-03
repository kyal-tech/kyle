#![allow(clippy::not_unsafe_ptr_arg_deref)]

pub mod memory;
pub mod io;
pub mod string;
pub mod list;
pub mod dict;
pub mod async_;
pub mod assert;
pub mod error;
pub mod panic;
pub mod platform;
pub mod net;
pub mod datetime;
pub mod uuid;
pub mod bytes;
pub mod set;
pub mod queue;
pub mod stack;
pub mod date;
pub mod decimal;
pub mod channel;
pub mod thread;
pub mod url;
pub mod regex;
pub mod crypto;
pub mod duration;
pub mod path_;
pub mod big_int;
pub mod deque;
pub mod linked_list;
pub mod sync;
pub mod log;
pub mod cli;
pub mod csv;

pub use memory::{ky_alloc, ky_free, ky_retain, ky_release};
pub use io::{ky_print, ky_println, ky_input, ky_input_with_prompt, ky_open, ky_read_str, ky_write_str, ky_close, ky_sleep, ky_now};
pub use string::{ky_i64_to_str, ky_f32_to_str, ky_str_to_i32, ky_str_to_i64, ky_strlen, ky_concat, ky_str_contains, ky_str_to_upper, ky_str_to_lower, ky_str_trim, ky_str_replace,
    ky_char_at, ky_char_to_str, ky_is_digit, ky_is_alpha, ky_is_alnum, ky_is_whitespace, ky_is_upper, ky_is_lower, ky_ord, ky_substr, ky_eq_str, ky_str_cmp, ky_clone_str, ky_clone_substr, ky_from_cstr, ky_getenv, ky_setenv,
    ky_str_builder_new, ky_str_builder_append, ky_str_builder_to_str, ky_str_builder_free, ky_str_index_of, ky_str_split, ky_str_to_list,
    str_builder_new, str_builder_append, str_builder_to_str, str_builder_free};
pub use list::{ky_list_new, ky_list_free, ky_list_push, ky_list_pop, ky_list_get, ky_list_set, ky_list_len, ky_list_reserve, ky_init_args, ky_list_index_of, ky_list_sort, ky_list_chunk};
pub use async_::{ky_spawn_task, ky_await_task, ky_yield};
pub use thread::{ky_spawn_thread, ky_join_thread};
pub use channel::{ky_channel_new, ky_channel_send, ky_channel_recv, ky_channel_close, ky_channel_len, ky_channel_free};
pub use dict::{ky_dict_new, ky_dict_free, ky_dict_get, ky_dict_set, ky_dict_len, ky_dict_contains, ky_dict_remove, ky_struct_to_json, ky_json_to_struct, ky_dict_values, ky_dict_items};
pub use dict::ky_dict_keys;
pub use net::{ky_tcp_listen, ky_tcp_accept, ky_tcp_read, ky_tcp_write, ky_tcp_close, ky_ptr_read_i32, ky_ptr_read_ptr, ky_ptr_write_i32, ky_ptr_write_ptr, ky_sha1, ky_base64_encode, ky_ws_accept, ky_ws_read_frame, ky_ws_send_frame};
pub use datetime::{ky_datetime_now, ky_datetime_parse, ky_datetime_format, ky_datetime_year, ky_datetime_month, ky_datetime_day, ky_datetime_hour, ky_datetime_minute, ky_datetime_second, ky_datetime_add_days, ky_datetime_add_hours, ky_datetime_diff, ky_datetime_from_ymdhms};
pub use uuid::{ky_uuid_v4, ky_uuid_parse};
pub use bytes::{ky_bytes_new, ky_bytes_free, ky_bytes_get, ky_bytes_set, ky_bytes_to_hex, ky_bytes_from_hex, ky_bytes_to_base64};
pub use set::{ky_set_new, ky_set_free, ky_set_add, ky_set_contains, ky_set_remove, ky_set_len, ky_set_clear, ky_set_union, ky_set_intersection, ky_set_difference, ky_set_symmetric_difference, ky_set_is_subset, ky_set_to_list};
pub use queue::{ky_queue_new, ky_queue_free, ky_queue_push, ky_queue_pop, ky_queue_peek, ky_queue_len, ky_queue_clear, ky_queue_to_list, ky_queue_to_stack};
pub use stack::{ky_stack_new, ky_stack_free, ky_stack_push, ky_stack_pop, ky_stack_peek, ky_stack_len, ky_stack_clear, ky_stack_to_list, ky_stack_to_queue};
pub use date::{ky_date_today, ky_date_from_ymd, ky_date_parse, ky_date_year, ky_date_month, ky_date_day, ky_date_weekday, ky_date_add_days, ky_date_format, ky_time_from_hms, ky_time_now, ky_time_parse, ky_time_hour, ky_time_minute, ky_time_second};
pub use decimal::{ky_decimal_from_str, ky_decimal_to_str, ky_decimal_round, ky_decimal_truncate};
pub use url::{ky_url_scheme, ky_url_host, ky_url_port, ky_url_path, ky_url_query};
pub use regex::{ky_regex_new, ky_regex_free, ky_regex_is_match, ky_regex_find, ky_regex_replace};
pub use crypto::{ky_sha256, ky_random_bytes};
pub use duration::{ky_duration_from_secs, ky_duration_from_millis, ky_duration_from_hours, ky_duration_from_days, ky_duration_to_str, ky_duration_free};
pub use path_::{ky_path_new, ky_path_dirname, ky_path_basename, ky_path_extension, ky_path_join, ky_path_to_str, ky_path_free};
pub use big_int::{ky_big_int_from_str, ky_big_int_from_i64, ky_big_int_add, ky_big_int_sub, ky_big_int_mul, ky_big_int_to_str, ky_big_int_free};
pub use deque::{ky_deque_new, ky_deque_free, ky_deque_push_back, ky_deque_push_front, ky_deque_pop_back, ky_deque_pop_front, ky_deque_peek_back, ky_deque_peek_front, ky_deque_len, ky_deque_clear};
pub use linked_list::{ky_linked_list_new, ky_linked_list_free, ky_linked_list_push_back, ky_linked_list_push_front, ky_linked_list_pop_back, ky_linked_list_pop_front, ky_linked_list_peek_back, ky_linked_list_peek_front, ky_linked_list_len, ky_linked_list_clear};

pub use sync::{ky_mutex_new, ky_mutex_lock, ky_mutex_store, ky_mutex_free,
    ky_atomic_i64_new, ky_atomic_i64_load, ky_atomic_i64_store, ky_atomic_i64_add, ky_atomic_i64_free,
    ky_atomic_bool_new, ky_atomic_bool_load, ky_atomic_bool_store, ky_atomic_bool_free};

pub use log::{ky_log_debug, ky_log_info, ky_log_warn, ky_log_error, ky_log_set_level, ky_log_set_output, ky_log_set_console};

pub use cli::{ky_cli_parse, ky_cli_arg, ky_cli_argc, ky_cli_has, ky_cli_get, ky_cli_get_int, ky_cli_get_bool, ky_cli_define, ky_cli_help};

pub use csv::{ky_csv_parse, ky_csv_free, ky_csv_row_count, ky_csv_col_count, ky_csv_get, ky_csv_get_col, ky_csv_to_str};

/// Power: compute base ** exp for i64 values. Returns i64 (truncated).
#[unsafe(no_mangle)]
pub extern "C" fn ky_pow(base: i64, exp: i64) -> i64 {
    if exp == 0 { return 1; }
    if exp < 0 { return 0; } // floor for negative exponents
    let mut result: i64 = 1;
    for _ in 0..exp {
        result = result.wrapping_mul(base);
    }
    result
}

/// `x +% p` = x + (x * p / 100)
#[unsafe(no_mangle)]
pub extern "C" fn ky_add_pct(x: i64, p: i64) -> i64 {
    x + (x * p / 100)
}

/// `x -% p` = x - (x * p / 100)  
#[unsafe(no_mangle)]
pub extern "C" fn ky_sub_pct(x: i64, p: i64) -> i64 {
    x - (x * p / 100)
}

/// `x *% p` = x * p / 100 (percentage of)
#[unsafe(no_mangle)]
pub extern "C" fn ky_mul_pct(x: i64, p: i64) -> i64 {
    x * p / 100
}
