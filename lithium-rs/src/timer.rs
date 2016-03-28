#[derive(PartialEq)]
pub struct TimerId(pub u32);

macro_rules! impl_timer {
    ($h:ident, $r:ident, $g:ident, $l:ident, $e:ident, $s:ident) => {

        use timer::TimerId;

        // Timer Abstraction --------------------------------------------------
        pub struct Timer<
            H: $h<R, G, L, E, S>,
            R: $r,
            G: $g<S, L, R>,
            L: $l<S>,
            E: $e,
            S: $s
        > {
            callbacks: BinaryHeap<TimerCallback<H, R, G, L, E, S>>,
            time: u64,
            id: u32
        }

        impl<
            H: $h<R, G, L, E, S>,
            R: $r,
            G: $g<S, L, R>,
            L: $l<S>,
            E: $e,
            S: $s

        > Timer<H, R, G, L, E, S> {

            pub fn new() -> Timer<H, R, G, L, E, S> {
                Timer {
                    callbacks: BinaryHeap::new(),
                    time: 0,
                    id: 0
                }
            }

            pub fn update(
                &mut self,
                dt: u64

            ) -> Vec<Box<FnMut(&mut H, Handle<H, R, G, L, E, S>)>> {

                self.time += dt;

                let mut callbacks = Vec::new();
                while {
                    self.callbacks.peek().map_or(false, |c| {
                        c.time <= self.time
                    })
                } {
                    // TODO check cancel list
                    callbacks.push(self.callbacks.pop().unwrap().func);
                }

                callbacks

            }

            pub fn schedule(
                &mut self,
                f: Box<FnMut(&mut H, Handle<H, R, G, L, E, S>)>,
                time: u64

            ) -> u32 {
                self.id += 1;
                self.callbacks.push(TimerCallback {
                    func: f,
                    time: self.time + time,
                    id: TimerId(self.id)
                });
                self.id
            }

            pub fn cancel(&mut self, _: TimerId) {
                // TODO push into cancel list
            }

        }


        // Timer Callback Wrapper ---------------------------------------------
        struct TimerCallback<
            H: $h<R, G, L, E, S>,
            R: $r,
            G: $g<S, L, R>,
            L: $l<S>,
            E: $e,
            S: $s
        > {
            func: Box<FnMut(&mut H, Handle<H, R, G, L, E, S>)>,
            time: u64,
            id: TimerId
        }

        impl<
            H: $h<R, G, L, E, S>,
            R: $r,
            G: $g<S, L, R>,
            L: $l<S>,
            E: $e,
            S: $s

        > Eq for TimerCallback<H, R, G, L, E, S> {}

        impl<
            H: $h<R, G, L, E, S>,
            R: $r,
            G: $g<S, L, R>,
            L: $l<S>,
            E: $e,
            S: $s

        > PartialEq for TimerCallback<H, R, G, L, E, S> {
            fn eq(&self, other: &TimerCallback<H, R, G, L, E, S>) -> bool {
                self.id == other.id
            }
        }

        impl<
            H: $h<R, G, L, E, S>,
            R: $r,
            G: $g<S, L, R>,
            L: $l<S>,
            E: $e,
            S: $s

        > Ord for TimerCallback<H, R, G, L, E, S> {
            // Explicitly implement the trait so the queue becomes a min-heap
            // instead of a max-heap.
            fn cmp(&self, other: &TimerCallback<H, R, G, L, E, S>) -> cmp::Ordering {
                other.time.cmp(&self.time)
            }
        }

        impl<
            H: $h<R, G, L, E, S>,
            R: $r,
            G: $g<S, L, R>,
            L: $l<S>,
            E: $e,
            S: $s

        > PartialOrd for TimerCallback<H, R, G, L, E, S> {
            fn partial_cmp(
                &self, other: &TimerCallback<H, R, G, L, E, S>

            ) -> Option<cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

    }
}

