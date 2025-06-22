# Nginx HTTP stub status module
An example of NGINX dynamic module that provides access to basic status information same as (ngx_stub_status)[https://nginx.org/en/docs/http/ngx_http_stub_status_module.html]. But it's implemented by (ngx-rust)[https://github.com/nginx/ngx-rust].

## Build

```
cargo build
```

## Example Configuration

```nginx configuration
load_module "modules/libngx_http_stub_status_module.so";

http {
    server {
        listen 8080;

        location / {
            stub_status on;
        }
    }
}
```

## Usage

1. Clone the git repository.
  ```
  % git clone git@github.com:u5surf/ngx_http_stub_status_module.git
  ```

2. Compile the module from the cloned repo.
  ```
  % cd ${CLONED_DIRECTORY}/ngx_http_stab_status_module
  % cargo build
  ```

3. Tweak building nginx to include ngx_stat_xxx symbols but not to include original ngx_http_stub_status_module in nginx auto/config.
  ```
  % git clone git@github.com:nginx/nginx
  % cd nginx
  # tweaking following the diff.
  % git diff
  diff --git a/auto/modules b/auto/modules
  index 38b3aba78..2d3dae14b 100644
  --- a/auto/modules
  +++ b/auto/modules
  @@ -942,11 +942,11 @@ if [ $HTTP = YES ]; then
       if [ $HTTP_STUB_STATUS = YES ]; then
           have=NGX_STAT_STUB . auto/have
   
  -        ngx_module_name=ngx_http_stub_status_module
  -        ngx_module_incs=
  -        ngx_module_deps=
  -        ngx_module_srcs=src/http/modules/ngx_http_stub_status_module.c
  -        ngx_module_libs=
  +        #ngx_module_name=ngx_http_stub_status_module
  +        #ngx_module_incs=
  +        #ngx_module_deps=
  +        #ngx_module_srcs=src/http/modules/ngx_http_stub_status_module.c
  +        #ngx_module_libs=
           ngx_module_link=$HTTP_STUB_STATUS
  diff --git a/auto/options b/auto/options
  index 6a6e990a0..559b98b9b 100644
  --- a/auto/options
  +++ b/auto/options
  @@ -109,7 +109,7 @@ HTTP_UPSTREAM_KEEPALIVE=YES
   HTTP_UPSTREAM_ZONE=YES
   
   # STUB
  -HTTP_STUB_STATUS=NO
  +HTTP_STUB_STATUS=YES

  % cd nginx
  % ./auto/configure
  % objdump -T ../nginx/objs/nginx | grep ngx_stat
  # to check whether ngx_stat symbols are included or not.
  
  % make
  % sudo make install
  ```

4. Add the `load_module` directive to your configuration.
  ```
  load_module "modules/libngx_http_stub_status_module.so";
  
  http {
      server {
          listen 8080;
  
          location /nginx_status {
              stub_status on;
          }
      }
  }
  ```

5. Start NGINX.
  ```
  % nginx -t && nginx
  ```

6. Test with `curl`
  ```
  % curl http://localhost:8080/nginx_status
  Active connections: 1
  server accepts handled requests 16 16 15
  Reading: 0 Writing: 1 Waiting: 0
  ```

## Caveats
This module is EXPERIMENTAL.
