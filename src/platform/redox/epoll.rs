use super::super::types::*;
use super::super::{Pal, PalEpoll};
use super::Sys;

use c_str::CStr;
use core::{mem, slice};
use fs::File;
use io::prelude::*;
use header::errno::*;
use header::fcntl::*;
use header::signal::sigset_t;
use header::sys_epoll::*;
use syscall::data::{Event, TimeSpec};
use syscall::flag::EVENT_READ;

impl PalEpoll for Sys {
    fn epoll_create1(flags: c_int) -> c_int {
        Sys::open(
            CStr::from_bytes_with_nul(b"event:\0").unwrap(),
            O_RDWR | flags,
            0
        )
    }

    fn epoll_ctl(epfd: c_int, op: c_int, fd: c_int, event: *mut epoll_event) -> c_int {
        let flags = unsafe { (*event).events };
        Sys::write(epfd, &Event {
            id: fd as usize,
            flags: flags as usize,
            data: event as usize
        }) as c_int
    }

    fn epoll_pwait(epfd: c_int, mut events: *mut epoll_event, maxevents: c_int, timeout: c_int, _sigset: *const sigset_t) -> c_int {
        // TODO: sigset

        let mut redox_events = vec![Event::default(); maxevents as usize];

        let _timer;
        if timeout != -1 {
            _timer = File::open(CStr::from_bytes_with_nul(b"time:\0").unwrap(), O_RDWR);
            match _timer {
                Err(_) => return -1,
                Ok(mut timer) => {
                    let mut time = TimeSpec::default();
                    if let Err(err) = timer.read(&mut time) {
                        return -1;
                    }
                    time.tv_nsec += timeout;
                    if let Err(err) = timer.write(&time) {
                        return -1;
                    }

                    if Sys::write(epfd, &Event {
                        id: timer.fd as usize,
                        flags: EVENT_READ,
                        data: 0
                    }) == -1 {
                        return -1;
                    }
                }
            }
        }

        let bytes_read = Sys::read(epfd, unsafe { slice::from_raw_parts_mut(
            redox_events.as_mut_ptr() as *mut u8,
            redox_events.len() * mem::size_of::<Event>()
        ) });
        if bytes_read == -1 {
            return -1;
        }
        let read = bytes_read as usize / mem::size_of::<Event>();

        for event in &redox_events {
            if event.data == 0 {
                return EINTR;
            }
            unsafe {
                *events = *(event.data as *mut epoll_event);
                events = events.add(mem::size_of::<epoll_event>());
            }
        }

        read as c_int
    }
}
