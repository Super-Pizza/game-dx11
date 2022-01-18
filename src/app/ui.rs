use super::*;

impl<'a> App<'a> {
    pub fn render_ui(&mut self) {
        match self.state {
            State::Loading => unimplemented!(),
            State::MainMenu(x) => match x {
                0 => {}
                _ => {
                    unimplemented!()
                }
            },
            State::InGame(u16) => todo!(),
        }
    }
}
