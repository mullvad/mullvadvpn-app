use super::oneshot_send;
use crate::{Daemon, Error, EventListener, ResponseTx};
use futures::future::Either;

impl<L: EventListener + Send + Clone + 'static> Daemon<L> {
    pub(super) fn on_get_www_auth_token(&mut self, tx: ResponseTx<String, Error>) {
        let fut = if let Some(account_token) = self.settings.get_account_token() {
            let future = self.account.get_www_auth_token(account_token);
            Either::Left(async move { future.await.map_err(Error::RestError) })
        } else {
            Either::Right(async { Err(Error::NoAccountToken) })
        };

        tokio::spawn(async move {
            oneshot_send(tx, fut.await, "get_www_auth_token response");
        });
    }
}
