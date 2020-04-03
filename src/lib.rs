#![deny(missing_docs)]
#![deny(unused_must_use)]
//! BoxFutures cannot be marked `#[must_use]` because they are just type
//! definitions. This newtype struct wraps a BoxFuture with something that
//! can be marked `#[must_use]`.
//!
//! # Will Not Compile:
//!
//! ```compile_fail
//! #![deny(unused_must_use)]
//!
//! use futures::future::FutureExt;
//!
//! #[tokio::main]
//! async fn main() {
//!     fn get_future() -> must_future::MustBoxFuture<'static, ()> {
//!         async { }.boxed().into()
//!     }
//!
//!     get_future(); // unused `must_future::MustBoxFuture` that must be used
//! }
//! ```

use futures::future::BoxFuture;

/// Wrap a future that may or may not be marked must_use with a newtype
/// that is marked must_use.
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct MustFuture<F: std::future::Future> {
    sub_fut: F,
}

impl<F: std::future::Future> MustFuture<F> {
    pin_utils::unsafe_pinned!(sub_fut: F);
}

impl<F: std::future::Future> From<F> for MustFuture<F> {
    fn from(f: F) -> Self {
        Self { sub_fut: f }
    }
}

impl<F: std::future::Future> std::future::Future for MustFuture<F> {
    type Output = F::Output;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context,
    ) -> std::task::Poll<Self::Output> {
        let p: std::pin::Pin<&mut F> = self.sub_fut();
        std::future::Future::poll(p, cx)
    }
}

impl<F: std::future::Future + std::marker::Unpin> std::marker::Unpin for MustFuture<F> {}

/// BoxFutures cannot be marked must_use because they are just type definitions.
/// This newtype struct wraps a BoxFuture with something that can be marked must_use.
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct MustBoxFuture<'lt, T> {
    sub_fut: BoxFuture<'lt, T>,
}

impl<'lt, T> From<BoxFuture<'lt, T>> for MustBoxFuture<'lt, T> {
    fn from(f: BoxFuture<'lt, T>) -> Self {
        Self { sub_fut: f }
    }
}

impl<'lt, T> std::future::Future for MustBoxFuture<'lt, T> {
    type Output = T;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context,
    ) -> std::task::Poll<Self::Output> {
        std::future::Future::poll(self.sub_fut.as_mut(), cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::future::FutureExt;

    #[tokio::test]
    pub async fn must_box_future_can_still_process() {
        fn get_future() -> MustBoxFuture<'static, &'static str> {
            async { "test1" }.boxed().into()
        }
        assert_eq!("test1", get_future().await,);
    }

    #[tokio::test]
    pub async fn must_future_can_still_process() {
        fn get_future() -> MustFuture<BoxFuture<'static, &'static str>> {
            async { "test2" }.boxed().into()
        }
        assert_eq!("test2", get_future().await,);
    }
}
