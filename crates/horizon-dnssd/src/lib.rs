//! One SRV question to the system resolver on iPhone and iPad. horizon-protocol asks the name servers in
//! resolv.conf itself, the way vanilla's JNDI client does, but an iOS app has no resolv.conf to read: without
//! this, `hypixel.net` was never redirected to its `_minecraft._tcp` target and the game knocked on the
//! website. dns_sd (`DNSServiceQueryRecord`, in libSystem) goes through mDNSResponder, which knows the
//! phone's name servers. The protocol forbids `unsafe`, so the C calls live here and it sees plain bytes.
//! Elsewhere there is nothing in the crate.

#[cfg(target_os = "ios")]
pub use ios::{TIMEOUT, query_srv};

#[cfg(target_os = "ios")]
mod ios {
    use std::{
        ffi::{CString, c_char, c_int, c_void},
        time::{Duration, Instant},
    };

    /// The wait for one answer: mDNSResponder asks each name server in turn itself, so this covers what
    /// horizon-protocol's one second per server does.
    pub const TIMEOUT: Duration = Duration::from_secs(2);

    const TYPE_SRV: u16 = 33;
    const CLASS_IN: u16 = 1;
    /// kDNSServiceFlagsMoreComing.
    const MORE_COMING: u32 = 0x1;
    /// kDNSServiceFlagsReturnIntermediates, so "no such record" arrives at once instead of as silence, and
    /// kDNSServiceFlagsTimeout.
    const FLAGS: u32 = 0x1000 | 0x10000;

    type Service = *mut c_void;
    type Reply = unsafe extern "C" fn(
        Service,
        u32,
        u32,
        i32,
        *const c_char,
        u16,
        u16,
        u16,
        *const c_void,
        u32,
        *mut c_void,
    );

    unsafe extern "C" {
        fn DNSServiceQueryRecord(
            service: *mut Service,
            flags: u32,
            interface: u32,
            name: *const c_char,
            kind: u16,
            class: u16,
            reply: Reply,
            context: *mut c_void,
        ) -> i32;
        fn DNSServiceRefSockFD(service: Service) -> c_int;
        fn DNSServiceProcessResult(service: Service) -> i32;
        fn DNSServiceRefDeallocate(service: Service);
    }

    #[derive(Default)]
    struct Answer {
        done: bool,
        rdata: Option<Vec<u8>>,
    }

    unsafe extern "C" fn reply(
        _: Service,
        flags: u32,
        _: u32,
        error: i32,
        _: *const c_char,
        kind: u16,
        _: u16,
        len: u16,
        data: *const c_void,
        _: u32,
        context: *mut c_void,
    ) {
        // SAFETY: `context` is `query_srv`'s `Answer`, alive until the service is deallocated, and only
        // touched here while `DNSServiceProcessResult` runs.
        let answer = unsafe { &mut *context.cast::<Answer>() };
        if error == 0 && kind == TYPE_SRV && answer.rdata.is_none() && !data.is_null() {
            // SAFETY: dns_sd hands `len` bytes of the record's rdata at `data`.
            answer.rdata =
                Some(unsafe { std::slice::from_raw_parts(data.cast::<u8>(), len.into()) }.to_vec());
        }
        if flags & MORE_COMING == 0 {
            answer.done = true;
        }
    }

    /// The rdata of the first SRV record for `name` (priority, weight, port, then the target's labels), or
    /// `None` for no record, an error, or no answer within `timeout`.
    pub fn query_srv(name: &str, timeout: Duration) -> Option<Vec<u8>> {
        let name = CString::new(name).ok()?;
        // Boxed, so the callback's pointer stays the one place the answer lives.
        let answer: *mut Answer = Box::into_raw(Box::default());
        let mut service: Service = std::ptr::null_mut();
        // SAFETY: the out-pointer and the name are valid for the call; `answer` outlives the service.
        let started = unsafe {
            DNSServiceQueryRecord(
                &mut service,
                FLAGS,
                0,
                name.as_ptr(),
                TYPE_SRV,
                CLASS_IN,
                reply,
                answer.cast(),
            )
        };
        if started == 0 {
            let deadline = Instant::now() + timeout;
            loop {
                // SAFETY: `answer` is only written inside `DNSServiceProcessResult`, on this thread.
                if unsafe { (*answer).done } {
                    break;
                }
                let left = deadline.saturating_duration_since(Instant::now());
                if left.is_zero() {
                    break;
                }
                let mut fd = libc::pollfd {
                    // SAFETY: `service` is live until the deallocation below.
                    fd: unsafe { DNSServiceRefSockFD(service) },
                    events: libc::POLLIN,
                    revents: 0,
                };
                let wait = left.as_millis().clamp(1, c_int::MAX as u128) as c_int;
                // SAFETY: one valid pollfd; the service is live.
                if unsafe { libc::poll(&mut fd, 1, wait) } <= 0
                    || unsafe { DNSServiceProcessResult(service) } != 0
                {
                    break;
                }
            }
            // SAFETY: deallocated once; no callback runs after it.
            unsafe { DNSServiceRefDeallocate(service) };
        }
        // SAFETY: from `Box::into_raw` above, and no callback can reach it any more.
        unsafe { Box::from_raw(answer) }.rdata
    }
}
