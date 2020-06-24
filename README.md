cmd_cache is a command line tool that runs a command and caches its output. If the output is not older than
the environment variable `CMD_CACHE_MAX_DAYS`, the command isn't run and the cached output is displayed instead.

It's handy to cache the output of long running or expansive commands without having to manually deal with temp files.

Simple usage that will display the same date until the cache expires

```
cmd_cache date
```

Real life example of a lazy admin that wants to grep the `dmesg` of a bunch of servers:
```
cat hosts | parallel cmd_cache ssh -n {} dmesg | grep -i segfault
cat hosts | parallel cmd_cache ssh -n {} dmesg | grep -i oom
[...]
```


`CMD_CACHE_MAX_DAYS` defaults to 7 days. It's a floating point number of days. Setting it to 0 forces the command to be run again and the cache to be refreshed.

Only `stdout` is cached, `stderr` is not captured but display when the command is first run.

The cached outputs are stored in `~/.cmd_cache` and are never removed
because cmd_cache is designed to run as fast as possible (walking a big directory is expansive)
and because one may run cmd_cache with different `CMD_CACHE_MAX_DAYS` values.

The cache directory may be cleaned with a simple:

```
find ~/.cmd_cache/ -type f -mtime +90 -delete
```

![Rust](https://github.com/bdejean/cmd_cache/workflows/Rust/badge.svg)
