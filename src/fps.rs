pub struct FpsCounter {
    last_time: Option<f64>,
    ticks_since_last: usize,
    ticks_per: usize,
}

impl FpsCounter {
    pub fn new(ticks_per: usize) -> Self {
        FpsCounter {
            last_time: None,
            ticks_since_last: 0,
            ticks_per
        }
    }

    pub fn tick(&mut self) -> Option<f64> {
        let now_time = web_sys::window().unwrap().performance().unwrap().now();

        match self.last_time {
            None => {
                self.last_time = Some(now_time);
                None
            }
            Some(t) => {
                self.ticks_since_last += 1;
                if self.ticks_per == self.ticks_since_last {
                    let elapsed = now_time - t;
                    self.last_time = Some(now_time);
                    self.ticks_since_last = 0;
                    Some(1000. * self.ticks_per as f64 / elapsed)
                } else {
                    None
                }
            }
        }
    }
}
