

use async_trait::async_trait;

pub mod tui;



pub enum UIControl{
    Message(String),
}

pub enum UIInput{
    Message(String),
    Close,
}

#[async_trait]
pub trait UI{
    fn send(&self, msg: UIControl);
    async fn send_async(&self, msg: UIControl);
    fn recv(&mut self) -> Option<UIInput>;
    async fn recv_async(&mut self) -> Option<UIInput>;

    fn shutdown(&mut self);
}










#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
