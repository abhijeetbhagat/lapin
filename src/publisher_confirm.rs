use crate::{
    message::BasicReturnMessage, pinky_swear::PinkySwear, returned_messages::ReturnedMessages,
};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

#[derive(Debug)]
pub struct PublisherConfirm {
    inner: Option<PinkySwear<Confirmation>>,
    returned_messages: ReturnedMessages,
    used: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Confirmation {
    Ack,
    Nack(BasicReturnMessage),
    NotRequested,
}

impl PublisherConfirm {
    pub(crate) fn new(
        inner: PinkySwear<Confirmation>,
        returned_messages: ReturnedMessages,
    ) -> Self {
        Self {
            inner: Some(inner),
            returned_messages,
            used: false,
        }
    }

    pub(crate) fn not_requested(returned_messages: ReturnedMessages) -> Self {
        Self {
            inner: Some(PinkySwear::new_with_data(Confirmation::NotRequested)),
            returned_messages,
            used: false,
        }
    }

    pub fn try_wait(&mut self) -> Option<Confirmation> {
        let confirmation = self
            .inner
            .as_ref()
            .expect("inner should only be None after Drop")
            .try_wait()?;
        self.used = true;
        Some(confirmation)
    }

    pub fn wait(&mut self) -> Confirmation {
        self.used = true;
        self.inner
            .as_ref()
            .expect("inner should only be None after Drop")
            .wait()
    }
}

impl Future for PublisherConfirm {
    type Output = Confirmation;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.as_mut();
        let res = Pin::new(
            &mut this
                .inner
                .as_mut()
                .expect("inner should only be None ater Drop"),
        )
        .poll(cx);
        if res.is_ready() {
            this.used = true;
        }
        res
    }
}

impl Drop for PublisherConfirm {
    fn drop(&mut self) {
        if !self.used {
            if let Some(promise) = self.inner.take() {
                self.returned_messages.register_dropped_confirm(promise);
            }
        }
    }
}
