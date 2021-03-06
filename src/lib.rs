#![doc(html_root_url = "https://docs.rs/mio/0.7.0")]
#![deny(missing_docs, missing_debug_implementations, rust_2018_idioms)]
// Disallow warnings when running tests.
#![cfg_attr(test, deny(warnings))]
// Disallow warnings in examples.
#![doc(test(attr(deny(warnings))))]

//! A fast, low-level IO library for Rust focusing on non-blocking APIs, event
//! notification, and other useful utilities for building high performance IO
//! apps.
//!
//! # Features
//!
//! * Non-blocking TCP, UDP
//! * I/O event queue backed by epoll, kqueue, and IOCP
//! * Zero allocations at runtime
//! * Platform specific extensions
//!
//! # Non-goals
//!
//! The following are specifically omitted from Mio and are left to the user or
//! higher-level libraries.
//!
//! * File operations
//! * Thread pools / multi-threaded event loop
//! * Timers
//!
//! # Platforms
//!
//! Currently supported platforms:
//!
//! * Android
//! * Bitrig
//! * DragonFly BSD
//! * FreeBSD
//! * Linux
//! * NetBSD
//! * OpenBSD
//! * Solaris
//! * Windows
//! * iOS
//! * macOS
//!
//! Mio can handle interfacing with each of the event systems of the
//! aforementioned platforms. The details of their implementation are further
//! discussed in [`Poll`].
//!
//! # Usage
//!
//! Using Mio starts by creating a [`Poll`], which reads events from the OS and
//! puts them into [`Events`]. You can handle IO events from the OS with it.
//!
//! For more detail, see [`Poll`].
//!
//! [`Poll`]: struct.Poll.html
//! [`Events`]: struct.Events.html
//!
//! # Example
//!
//! ```
//! use mio::*;
//! use mio::net::{TcpListener, TcpStream};
//!
//! // Setup some tokens to allow us to identify which event is
//! // for which socket.
//! const SERVER: Token = Token(0);
//! const CLIENT: Token = Token(1);
//!
//! let addr = "127.0.0.1:13265".parse().unwrap();
//!
//! // Setup the server socket
//! let server = TcpListener::bind(addr).unwrap();
//!
//! // Create a poll instance
//! let mut poll = Poll::new().unwrap();
//! let registry = poll.registry().clone();
//!
//! // Start listening for incoming connections
//! registry.register(&server, SERVER, Interests::READABLE).unwrap();
//!
//! // Setup the client socket
//! let sock = TcpStream::connect(addr).unwrap();
//!
//! // Register the socket
//! registry.register(&sock, CLIENT, Interests::READABLE).unwrap();
//!
//! // Create storage for events
//! let mut events = Events::with_capacity(1024);
//!
//! loop {
//!     poll.poll(&mut events, None).unwrap();
//!
//!     for event in events.iter() {
//!         match event.token() {
//!             SERVER => {
//!                 // Accept and drop the socket immediately, this will close
//!                 // the socket and notify the client of the EOF.
//!                 let _ = server.accept();
//!             }
//!             CLIENT => {
//!                 // The server just shuts down the socket, let's just exit
//!                 // from our event loop.
//!                 return;
//!             }
//!             _ => unreachable!(),
//!         }
//!     }
//! }
//!
//! ```

mod interests;
mod poll;
mod sys;
mod token;
mod waker;

pub mod event;
pub mod net;

pub use event::Events;
pub use interests::Interests;
pub use poll::{Poll, Registry};
pub use token::Token;
pub use waker::Waker;

#[cfg(unix)]
pub mod unix {
    //! Unix only extensions.

    pub use crate::sys::SourceFd;
}

/// Windows-only extensions to the mio crate.
///
/// Mio on windows is currently implemented with IOCP for a high-performance
/// implementation of asynchronous I/O. Mio then provides TCP and UDP as sample
/// bindings for the system to connect networking types to asynchronous I/O. On
/// Unix this scheme is then also extensible to all other file descriptors with
/// the `SourceFd` type, but on Windows no such analog is available. The
/// purpose of this module, however, is to similarly provide a mechanism for
/// foreign I/O types to get hooked up into the IOCP event loop.
///
/// This module provides two types for interfacing with a custom IOCP handle:
///
/// * `Binding` - this type is intended to govern binding with mio's `Poll`
///   type. Each I/O object should contain an instance of `Binding` that's
///   interfaced with for the implementation of the `Source` trait. The
///   `register`, `reregister`, and `deregister` methods for the `Source` trait
///   all have rough analogs with `Binding`.
///
///   Note that this type **does not handle readiness**. That is, this type does
///   not handle whether sockets are readable/writable/etc. It's intended that
///   IOCP types will internally manage this state with a `SetReadiness` type
///   from the `poll` module. The `SetReadiness` is typically lazily created on
///   the first time that `Source::register` is called and then stored in the
///   I/O object.
///
///   Also note that for types which represent streams of bytes the mio
///   interface of *readiness* doesn't map directly to the Windows model of
///   *completion*. This means that types will have to perform internal
///   buffering to ensure that a readiness interface can be provided. For a
///   sample implementation see the TCP/UDP modules in mio itself.
///
/// * `Overlapped` - this type is intended to be used as the concrete instances
///   of the `OVERLAPPED` type that most win32 methods expect. It's crucial, for
///   safety, that all asynchronous operations are initiated with an instance of
///   `Overlapped` and not another instantiation of `OVERLAPPED`.
///
///   Mio's `Overlapped` type is created with a function pointer that receives
///   a `OVERLAPPED_ENTRY` type when called. This `OVERLAPPED_ENTRY` type is
///   defined in the `winapi` crate. Whenever a completion is posted to an IOCP
///   object the `OVERLAPPED` that was signaled will be interpreted as
///   `Overlapped` in the mio crate and this function pointer will be invoked.
///   Through this function pointer, and through the `OVERLAPPED` pointer,
///   implementations can handle management of I/O events.
///
/// When put together these two types enable custom Windows handles to be
/// registered with mio's event loops. The `Binding` type is used to associate
/// handles and the `Overlapped` type is used to execute I/O operations. When
/// the I/O operations are completed a custom function pointer is called which
/// typically modifies a `SetReadiness` set by `Source` methods which will get
/// later hooked into the mio event loop.
#[cfg(windows)]
pub mod windows {
    pub use crate::sys::{Binding, Overlapped};
}
