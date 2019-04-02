use std::sync::{Arc, Mutex};
use tok::prelude::*;
use tok::sync::{mpsc, oneshot};
use crate::{Error, Context, Answer, AsyncID};

// TODO: these are probably a little magical
impl From<tok::sync::oneshot::error::RecvError> for Error {
    fn from(_err: tok::sync::oneshot::error::RecvError) -> Self {
        Error::LostContextFuture
    }
}

impl From<tok::sync::mpsc::error::SendError> for Error {
    fn from(_err: tok::sync::mpsc::error::SendError) -> Self {
        Error::LostContextFuture
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct Question {
    name: Arc<str>,
    class: u16,
    ty: u16,
}

#[derive(Debug, Clone)]
pub struct ContextFutureHandle {
    ch: mpsc::Sender<CtlMsg>,
}

impl ContextFutureHandle {
    pub fn resolve(&mut self, name: &str, class: u16, ty: u16) -> AnswerFuture {
        let ctl_ch = self.ch.clone();
        let name = name.into();
        let q = Question { name, class, ty };
        let state = AnswerFutureState::Initial { q };
        AnswerFuture { ctl_ch, state }
    }
}

#[derive(Debug)]
enum CtlMsg {
    QueryNew {
        question: Question,
        ch: oneshot::Sender<Result<(AsyncID, oneshot::Receiver<Result<Answer, Error>>), Error>>,
    },
    QueryCancel(AsyncID),
}

#[derive(Debug)]
#[must_use = "futures do nothing unless polled"]
pub struct AnswerFuture {
    ctl_ch: mpsc::Sender<CtlMsg>,
    state: AnswerFutureState,
}

#[derive(Debug)]
enum AnswerFutureState {
    Initial {
        q: Question,
    },
    Id {
        ctl_complete: bool,
        ch: oneshot::Receiver<Result<(AsyncID, oneshot::Receiver<Result<Answer, Error>>), Error>>,
    },
    Answer {
        id: AsyncID,
        ch: oneshot::Receiver<Result<Answer, Error>>,
    },
    Done,
}

impl Drop for AnswerFuture {
    fn drop(&mut self) {
        let mut state = AnswerFutureState::Done;
        std::mem::swap(&mut self.state, &mut state);
        let id = match state {
            AnswerFutureState::Id { mut ch, .. } => match ch.poll() {
                Ok(Async::Ready(Ok((id, _an_ch)))) => id,
                _ => return,
            },
            AnswerFutureState::Answer { id, mut ch } => match ch.poll() {
                Ok(Async::NotReady) => id,
                _ => return,
            },
            _ => return,
        };
        let f = self
            .ctl_ch
            .clone()
            .send(CtlMsg::QueryCancel(id))
            .map(drop)
            .map_err(drop);
        tok::spawn(f);
    }
}

impl futures::Future for AnswerFuture {
    type Item = Answer;
    type Error = Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            match self.state {
                AnswerFutureState::Initial { ref q } => {
                    futures::try_ready!(self.ctl_ch.poll_ready());
                    let (tx, rx) = oneshot::channel();
                    let msg = CtlMsg::QueryNew {
                        question: q.clone(),
                        ch: tx,
                    };
                    match self.ctl_ch.start_send(msg) {
                        Ok(AsyncSink::Ready) => {
                            let mut new_state = AnswerFutureState::Id {
                                ch: rx,
                                ctl_complete: false,
                            };
                            std::mem::swap(&mut self.state, &mut new_state);
                        }
                        Ok(AsyncSink::NotReady(_)) => return Ok(Async::NotReady),
                        Err(err) => {
                            std::mem::swap(&mut self.state, &mut AnswerFutureState::Done);
                            return Err(err.into());
                        }
                    }
                }
                AnswerFutureState::Id {
                    ref mut ctl_complete,
                    ..
                } if !*ctl_complete => match self.ctl_ch.poll_complete() {
                    Ok(Async::Ready(())) => *ctl_complete = true,
                    Ok(Async::NotReady) => return Ok(Async::NotReady),
                    Err(err) => {
                        std::mem::swap(&mut self.state, &mut AnswerFutureState::Done);
                        return Err(err.into());
                    }
                },
                AnswerFutureState::Id { ref mut ch, .. } => match ch.poll() {
                    Ok(Async::Ready(r)) => match r {
                        Ok((id, ch)) => {
                            let mut new_state = AnswerFutureState::Answer { id, ch };
                            std::mem::swap(&mut self.state, &mut new_state);
                        }
                        Err(err) => {
                            std::mem::swap(&mut self.state, &mut AnswerFutureState::Done);
                            return Err(err.into());
                        }
                    },
                    Ok(Async::NotReady) => {
                        return Ok(Async::NotReady);
                    }
                    Err(err) => {
                        std::mem::swap(&mut self.state, &mut AnswerFutureState::Done);
                        return Err(err.into());
                    }
                },
                AnswerFutureState::Answer { ref mut ch, .. } => match ch.poll() {
                    Ok(Async::Ready(r)) => {
                        std::mem::swap(&mut self.state, &mut AnswerFutureState::Done);
                        return r.map(Async::Ready).map_err(Into::into);
                    }
                    Ok(Async::NotReady) => return Ok(Async::NotReady),
                    Err(err) => {
                        std::mem::swap(&mut self.state, &mut AnswerFutureState::Done);
                        return Err(err.into());
                    }
                },
                AnswerFutureState::Done => unreachable!(),
            }
        }
    }
}

#[must_use = "futures do nothing unless polled"]
pub struct ContextFuture {
    context: tok::reactor::PollEvented2<Context>,
    ctl_rx: mpsc::Receiver<CtlMsg>,
    pending: Arc<Mutex<Vec<(Question, AsyncID, oneshot::Sender<Result<Answer, Error>>)>>>,
}

impl ContextFuture {
    pub fn from_context(context: Context) -> (ContextFutureHandle, Self) {
        let (ctl_tx, ctl_rx) = mpsc::channel(5);
        let handle = ContextFutureHandle { ch: ctl_tx };
        let context = tok::reactor::PollEvented2::new(context);
        let pending = Default::default();
        let resolver = ContextFuture {
            context,
            ctl_rx,
            pending,
        };
        (handle, resolver)
    }
    fn handle_ctl_msg(&mut self, msg: CtlMsg) {
        match msg {
            CtlMsg::QueryNew { question, ch } => {
                let mut pending = self.pending.lock().expect("lock pending");
                let pending_arc = Arc::clone(&self.pending);
                let f = move |id, r| {
                    let mut pending = pending_arc.lock().expect("lock");
                    if let Some(position) = pending.iter().position(|&(_, pid, _)| pid == id) {
                        let (_, _, target) = pending.swap_remove(position);
                        let _ = target.send(r);
                    }
                };
                match self.context.get_ref().resolve_async(
                    &question.name,
                    question.ty,
                    question.class,
                    f,
                ) {
                    Ok(id) => {
                        let (tx, rx) = oneshot::channel();
                        if ch.send(Ok((id, rx))).is_err() {
                            self.context.get_ref().cancel(id);
                        } else {
                            pending.push((question, id, tx));
                        }
                    }
                    Err(err) => {
                        let _ = ch.send(Err(err));
                    }
                }
            }
            CtlMsg::QueryCancel(id) => {
                let mut pending = self.pending.lock().unwrap();
                if let Some(position) = pending.iter().position(|&(_, pid, _)| pid == id) {
                    pending.swap_remove(position);
                    self.context.get_ref().cancel(id);
                }
            }
        }
    }
}

impl Future for ContextFuture {
    type Item = ();
    type Error = ();
    fn poll(&mut self) -> Poll<(), ()> {
        let ctl_done = loop {
            match self.ctl_rx.poll() {
                Ok(Async::Ready(Some(msg))) => self.handle_ctl_msg(msg),
                Ok(Async::Ready(None)) => break true,
                Ok(Async::NotReady) => break false,
                Err(_) => {
                    // TODO: log error? continue?
                    return Err(());
                }
            }
        };
        let ready = mio::Ready::readable();
        if self
            .context
            .poll_read_ready(ready)
            .map_err(drop)?
            .is_ready()
        {
            self.context.clear_read_ready(ready).unwrap(); // TODO: fixme
            self.context.get_ref().process().unwrap(); // TODO: fixme
        }
        if ctl_done && self.pending.lock().unwrap().is_empty() {
            return Ok(Async::Ready(()));
        }
        Ok(Async::NotReady)
    }
}
