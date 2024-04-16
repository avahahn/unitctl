# Unitctl
A command line utility to access and control running instances of NGINX Unit.

# Building
First build with `cargo build`. Then copy `target/debug/unitctl` to wherever you want.

# Usage
```
λ target/debug/unitctl

Usage: unitctl <COMMAND>

Commands:
  start   
  status  
  api     
  schema  
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```
```
λ target/debug/unitctl start

Usage: unitctl start --socket <SOCKET>

Options:
  -s, --socket <SOCKET>  path to desired control socket
  -h, --help             Print help
```
```
λ target/debug/unitctl status

Usage: unitctl status [OPTIONS] --uri <URI>

Options:
  -u, --uri <URI>        URI the control API listens on
  -s, --socket <SOCKET>  Unix Socket the control API listens on
  -v, --verbose          switch to trigger verbose behavior in libcurl
  -h, --help             Print help
```
```
λ target/debug/unitctl api

Usage: unitctl api [OPTIONS] --uri <URI>

Options:
  -u, --uri <URI>        URI for API operation
  -s, --socket <SOCKET>  Unix Socket the control API listens on
  -j, --json <JSON>      inline JSON data to post to API
  -f, --file <FILE>      file containing data to post to API.
  -d, --delete           switch to trigger a delete operation on an API endpoint.
  -v, --verbose          switch to trigger verbose behavior in libcurl
  -h, --help             Print help
```
```
λ target/debug/unitctl schema

Usage: unitctl schema --path <PATH>

Options:
  -p, --path <PATH>  path for schema query
  -h, --help         Print help
```

# Examples
Getting started:
```
λ target/debug/unitctl start -s /tmp

Using default tag: latest
latest: Pulling from library/unit
Digest: sha256:bae510e7594ba1a68895859cc7fa79ed4231907820d46c4a859f5cbe25f85a7e
Status: Image is up to date for unit:latest
docker.io/library/unit:latest
e22783542fabc49a120edc84fc3a48830b8fec0bda16ab20ed0fac95c743486b
Congratulations! NGINX Unit now running at /tmp/control.unit.sock
NOTICE: Socket access is root only by default. Run chown.
Current directory mounted to /www in NGINX Unit container.
```

Healthy Status:
```
λ target/debug/unitctl status -s /tmp/control.unit.sock -u 'http://localhost/' -v

*   Trying /tmp/control.unit.sock:0...
* Connected to localhost (/tmp/control.unit.sock) port 0
> GET / HTTP/1.1
Host: localhost
Accept: */*

* Request completely sent off
< HTTP/1.1 200 OK
< Server: Unit/1.32.1
< Date: Tue, 16 Apr 2024 00:32:37 GMT
< Content-Type: application/json
< Content-Length: 595
< Connection: close
< 
{ {
        "certificates": {},
        "js_modules": {},
        "config": {
                "listeners": {
                        "*:80": {
                                "pass": "routes"
                        }
                },

                "routes": [
                        {
                                "match": {
                                        "headers": {
                                                "accept": "*text/html*"
                                        }
                                },

                                "action": {
                                        "share": "/usr/share/unit/welcome/welcome.html"
                                }
                        },
                        {
                                "action": {
                                        "share": "/usr/share/unit/welcome/welcome.md"
                                }
                        }
                ]
        },

        "status": {
                "connections": {
                        "accepted": 0,
                        "active": 0,
                        "idle": 0,
                        "closed": 0
                },

                "requests": {
                        "total": 0
                },

                "applications": {}
        }
}
* Closing connection
```

Querying the API:
```
λ target/debug/unitctl api -s /tmp/control.unit.sock -u 'http://localhost/config' -v

*   Trying /tmp/control.unit.sock:0...
* Connected to localhost (/tmp/control.unit.sock) port 0
> GET /config HTTP/1.1
Host: localhost
Accept: */*

* Request completely sent off
< HTTP/1.1 200 OK
< Server: Unit/1.32.1
< Date: Tue, 16 Apr 2024 00:34:31 GMT
< Content-Type: application/json
< Content-Length: 335
< Connection: close
< 
{ {
        "listeners": {
                "*:80": {
                        "pass": "routes"
                }
        },

        "routes": [
                {
                        "match": {
                                "headers": {
                                        "accept": "*text/html*"
                                }
                        },

                        "action": {
                                "share": "/usr/share/unit/welcome/welcome.html"
                        }
                },
                {
                        "action": {
                                "share": "/usr/share/unit/welcome/welcome.md"
                        }
                }
        ]
}
* Closing connection
```

Getting API Documentation:
```
λ target/debug/unitctl schema -p /status/connections/idle

---
summary: "Endpoint for the `idle` connections number"
get:
  operationId: getStatusConnectionsIdle
  summary: Retrieve the idle connections number
  description: "Retrieves the `idle` connections number that represents Unit's [connection statistics](https://unit.nginx.org/usagestats/)."
  tags:
    - status
  responses:
    "200":
      description: "OK; the `idle` number exists in the configuration."
      content:
        application/json:
          schema:
            type: integer
          examples:
            Idle:
              value: 4
```