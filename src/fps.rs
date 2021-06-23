const MILISECONDS_PER_SECOND: f64 = 1000.;

/// Lightweight counter for measuring frames per second using the browser's `Performance` API, if
/// available.
///
/// The API is designed around use-cases like logging to console.log, rather than a permanent
/// on-screen FPS counter.
pub struct FpsCounter {
    last_time: Option<f64>,
    ticks_since_last: usize,
    ticks_per: usize,
}

impl FpsCounter {
    /// Construct an object for measuring frames per second. `ticks_per` determines how many frames
    /// the FPS is measured over.
    pub fn new(ticks_per: usize) -> Self {
        FpsCounter {
            last_time: None,
            ticks_since_last: 0,
            ticks_per,
        }
    }

    /// Count a frame. Every `self.ticks_per` calls to this method, the value `Some(fps)` is
    /// returned where `fps` is the frames per second over the last `ticks_per` frames. Every other
    /// call to `tick` returns `None`.
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
                    Some(MILISECONDS_PER_SECOND * self.ticks_per as f64 / elapsed)
                } else {
                    None
                }
            }
        }
    }
}
