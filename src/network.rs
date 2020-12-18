use std::rc::Rc;
use std::sync::RwLock;

pub struct Network<T> {
    rxers: RwLock<Vec<Rc<Receiver<T>>>>,
}

impl<T> Network<T> {
    pub fn new() -> Rc<Network<T>> {
        Rc::new(Network {
            rxers: RwLock::new(Vec::new()),
        })
    }

    pub fn new_sender(self: &Rc<Network<T>>) -> Sender<T> {
        Sender {
            network: self.clone()
        }
    }

    pub fn new_receiver(self: &Rc<Network<T>>) -> Rc<Receiver<T>> {
        let rx = Rc::new(Receiver {
            queue: RwLock::new(Vec::new()),
        });
        {
            let mut rxers = self.rxers.write().unwrap();
            rxers.push(rx.clone());
        }
        rx
    }

    pub fn send(self: &Rc<Network<T>>, data: T) {
        let data = Rc::new(data);
        let rxers = {
            self.rxers.read().unwrap().clone()
        };
        for rx in rxers {
            let mut queue = rx.queue.write().unwrap();
            queue.push(data.clone());
        }
    }
}

pub struct Sender<T> {
    network: Rc<Network<T>>,
}

impl<T> Sender<T> {
    // Note to self. Could remove some locks if we had a buffer in the sender
    // that only forwards a batch of messages at a time. Would maybe have to
    // bubble update up from rxers to clear sender queus or something.
    pub fn send(&self, data: T) {
        self.network.send(data);
    }
}

pub struct Receiver<T>
{
    queue: RwLock<Vec<Rc<T>>>,
}

impl<T> Receiver<T> {
    pub fn read(&self) -> Vec<Rc<T>> {
        let mut queue = self.queue.write().unwrap();
        let ret = queue.clone();
        queue.clear();
        ret
    }
}
