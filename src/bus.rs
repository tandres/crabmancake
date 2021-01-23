use std::rc::Rc;
use std::sync::RwLock;

pub type Bus<T> = Rc<BusInner<T>>;

#[allow(dead_code)]
pub struct BusInner<T> {
    rxers: RwLock<Vec<Receiver<T>>>,
}

pub fn create_bus<T>() -> Bus<T> {
    Rc::new(BusInner {
        rxers: RwLock::new(Vec::new()),
    })
}

impl<T> BusInner<T> {
    #[allow(dead_code)]
    pub fn new_sender(self: &Rc<BusInner<T>>) -> Sender<T> {
        Sender {
            bus: self.clone()
        }
    }

    #[allow(dead_code)]
    pub fn new_receiver(self: &Rc<BusInner<T>>) -> Receiver<T> {
        let rx = Rc::new(ReceiverInner {
            queue: RwLock::new(Vec::new()),
        });
        {
            let mut rxers = self.rxers.write().unwrap();
            rxers.push(rx.clone());
        }
        rx
    }

    #[allow(dead_code)]
    pub fn send(self: &Rc<BusInner<T>>, data: T) {
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

#[allow(dead_code)]
pub struct Sender<T> {
    bus: Bus<T>,
}

impl<T> Sender<T> {
    // Note to self. Could remove some locks if we had a buffer in the sender
    // that only forwards a batch of messages at a time. Would maybe have to
    // bubble update up from rxers to clear sender queus or something.
    #[allow(dead_code)]
    pub fn send(&self, data: T) {
        self.bus.send(data);
    }
}

pub type Receiver<T> = Rc<ReceiverInner<T>>;

pub struct ReceiverInner<T> {
    queue: RwLock<Vec<Rc<T>>>,
}

impl<T> ReceiverInner<T> {
    #[allow(dead_code)]
    pub fn read(&self) -> Vec<Rc<T>> {
        let mut queue = self.queue.write().unwrap();
        let ret = queue.clone();
        queue.clear();
        ret
    }
}
