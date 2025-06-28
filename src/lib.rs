use ngx::core::Buffer;
use ngx::ffi::{
    nginx_version, ngx_atomic_t, ngx_chain_t, ngx_command_t, ngx_conf_t, ngx_http_add_variable,
    ngx_http_module_t, ngx_http_variable_t, ngx_int_t, ngx_module_t, ngx_uint_t,
    ngx_variable_value_t, NGX_CONF_NOARGS, NGX_CONF_TAKE1, NGX_HTTP_LOC_CONF,
    NGX_HTTP_LOC_CONF_OFFSET, NGX_HTTP_MODULE, NGX_HTTP_SRV_CONF, NGX_RS_MODULE_SIGNATURE,
};
use ngx::http::{HttpModule, HttpModuleLocationConf};
use ngx::{core, http};
use ngx::core::NgxString;
use ngx::{http_request_handler, http_variable_get, ngx_modules, ngx_string};
use std::os::raw::{c_char, c_void};

struct Module;

#[no_mangle]
static mut ngx_http_stub_status_commands: [ngx_command_t; 2] = [
    ngx_command_t {
        name: ngx_string!("stub_status"),
        type_: (NGX_HTTP_SRV_CONF | NGX_HTTP_LOC_CONF | NGX_CONF_NOARGS | NGX_CONF_TAKE1)
            as ngx_uint_t,
        set: Some(ngx_http_set_stub_status),
        conf: NGX_HTTP_LOC_CONF_OFFSET,
        offset: 0,
        post: std::ptr::null_mut(),
    },
    ngx_command_t::empty(),
];

#[no_mangle]
static ngx_http_stub_status_module_ctx: ngx_http_module_t = ngx_http_module_t {
    preconfiguration: Some(Module::preconfiguration), // ngx_http_stub_status_add_variables
    postconfiguration: None,
    create_main_conf: None,
    init_main_conf: None,
    create_srv_conf: None,
    merge_srv_conf: None,
    create_loc_conf: None,
    merge_loc_conf: None,
};

ngx_modules!(ngx_http_stub_status_module);

#[no_mangle]
pub static mut ngx_http_stub_status_module: ngx_module_t = ngx_module_t {
    ctx_index: ngx_uint_t::max_value(),
    index: ngx_uint_t::max_value(),
    name: std::ptr::null_mut(),
    spare0: 0,
    spare1: 0,
    version: nginx_version as ngx_uint_t,
    signature: NGX_RS_MODULE_SIGNATURE.as_ptr() as *const c_char,

    ctx: &ngx_http_stub_status_module_ctx as *const _ as *mut _,
    commands: unsafe { &ngx_http_stub_status_commands[0] as *const _ as *mut _ },
    type_: NGX_HTTP_MODULE as ngx_uint_t,

    init_master: None,
    init_module: None,
    init_process: None,
    init_thread: None,
    exit_thread: None,
    exit_process: None,
    exit_master: None,

    spare_hook0: 0,
    spare_hook1: 0,
    spare_hook2: 0,
    spare_hook3: 0,
    spare_hook4: 0,
    spare_hook5: 0,
    spare_hook6: 0,
    spare_hook7: 0,
};

static mut NGX_HTTP_STUB_STATUS_VARS: [ngx_http_variable_t; 4] = [
    ngx_http_variable_t {
        name: ngx_string!("connections_active"),
        set_handler: None,
        get_handler: Some(ngx_http_stub_status_variable),
        data: 0,
        flags: 2,
        index: 0,
    },
    ngx_http_variable_t {
        name: ngx_string!("connections_reading"),
        set_handler: None,
        get_handler: Some(ngx_http_stub_status_variable),
        data: 1,
        flags: 2,
        index: 0,
    },
    ngx_http_variable_t {
        name: ngx_string!("connections_writing"),
        set_handler: None,
        get_handler: Some(ngx_http_stub_status_variable),
        data: 2,
        flags: 2,
        index: 0,
    },
    ngx_http_variable_t {
        name: ngx_string!("connections_waiting"),
        set_handler: None,
        get_handler: Some(ngx_http_stub_status_variable),
        data: 3,
        flags: 2,
        index: 0,
    },
];

unsafe extern "C" {
    // const instead of must just because we don't need to modify them
    static ngx_stat_accepted: *const ngx_atomic_t;
    static ngx_stat_handled: *const ngx_atomic_t;
    static ngx_stat_active: *const ngx_atomic_t;
    static ngx_stat_requests: *const ngx_atomic_t;
    static ngx_stat_reading: *const ngx_atomic_t;
    static ngx_stat_writing: *const ngx_atomic_t;
    static ngx_stat_waiting: *const ngx_atomic_t;
}

http_request_handler!(
    ngx_http_stub_status_handler,
    |request: &mut http::Request| {
        let body = format!(
            r"Active connections: {ac}
server accepts handled requests {ap} {hn} {rq}
Reading: {rd} Writing: {wr} Waiting: {wt}
",
            ac = unsafe { *ngx_stat_active },
            ap = unsafe { *ngx_stat_accepted },
            hn = unsafe { *ngx_stat_handled },
            rq = unsafe { *ngx_stat_requests },
            rd = unsafe { *ngx_stat_reading },
            wr = unsafe { *ngx_stat_writing },
            wt = unsafe { *ngx_stat_waiting },
        );
        let mut buf = match request.pool().create_buffer_from_str(&body) {
            Some(buf) => buf,
            None => return http::HTTPStatus::INTERNAL_SERVER_ERROR.into(),
        };

        request.set_content_length_n(buf.len());
        request.set_status(http::HTTPStatus::OK);

        buf.set_last_buf(request.is_main());
        buf.set_last_in_chain(true);

        let rc = request.send_header();
        if rc == core::Status::NGX_ERROR || rc > core::Status::NGX_OK || request.header_only() {
            return rc;
        }

        let mut out = ngx_chain_t {
            buf: buf.as_ngx_buf_mut(),
            next: std::ptr::null_mut(),
        };
        return request.output_filter(&mut out);
    }
);

http_variable_get!(
    ngx_http_stub_status_variable,
    |req: &mut http::Request, v: *mut ngx_variable_value_t, data: usize| {
        use std::fmt::Write;

        let mut str = NgxString::new_in(req.pool());
        let _ = match data {
            0 => write!(str, "{ac}", ac = unsafe { *ngx_stat_active }),
            1 => write!(str, "{rd}", rd = unsafe { *ngx_stat_reading }),
            2 => write!(str, "{wr}", wr = unsafe { *ngx_stat_writing }),
            3 => write!(str, "{wt}", wt = unsafe { *ngx_stat_waiting }),
            _ => write!(str, "0"),
        };
        let (data, len, _, _) = str.into_raw_parts();
        (*v).set_valid(1);
        (*v).set_no_cacheable(0);
        (*v).set_not_found(0);
        (*v).set_len(len as _);
        (*v).data = data;
        core::Status::NGX_OK
    }
);

impl http::HttpModule for Module {
    fn module() -> &'static ngx_module_t {
        unsafe { &*::core::ptr::addr_of!(ngx_http_stub_status_module) }
    }
    // ngx_http_stub_status_add_variables
    unsafe extern "C" fn preconfiguration(cf: *mut ngx_conf_t) -> ngx_int_t {
        for mut v in NGX_HTTP_STUB_STATUS_VARS {
            let var = ngx_http_add_variable(cf, &mut v.name, v.flags);
            if var.is_null() {
                return core::Status::NGX_ERROR.into();
            }
            (*var).get_handler = v.get_handler;
            (*var).data = v.data;
        }
        if unsafe {
            ngx_stat_accepted.is_null()
                || ngx_stat_handled.is_null()
                || ngx_stat_active.is_null()
                || ngx_stat_requests.is_null()
                || ngx_stat_reading.is_null()
                || ngx_stat_writing.is_null()
                || ngx_stat_waiting.is_null()
        } {
            return core::Status::NGX_ERROR.into();
        }
        core::Status::NGX_OK.into()
    }
}

#[no_mangle]
extern "C" fn ngx_http_set_stub_status(
    cf: *mut ngx_conf_t,
    _cmd: *mut ngx_command_t,
    _conf: *mut c_void,
) -> *mut c_char {
    let cf = unsafe { &mut *cf };
    let clcf = http::NgxHttpCoreModule::location_conf_mut(cf).expect("core location conf");
    clcf.handler = Some(ngx_http_stub_status_handler);
    ngx::core::NGX_CONF_OK
}
