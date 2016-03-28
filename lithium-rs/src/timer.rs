macro_rules! impl_timer {
    ($e:ident, $s:ident, $b:ident, $r:ident, $h:ident, $g:ident) => {

        // Timer Abstraction --------------------------------------------------
        pub struct Timer<E: $e, S: $s, L: $b<S>, R: $r, H: $h<E, S, L, R, G>, G: $g<S, L, R>> {
            callbacks: BinaryHeap<TimerCallback<E, S, L, R, H, G>>,
            time: u64,
            id: u32
        }

        impl<
            E: $e,
            S: $s,
            L: $b<S>,
            R: $r,
            H: $h<E, S, L, R, G>,
            G: $g<S, L, R>

        > Timer<E, S, L, R, H, G> {

            pub fn new() -> Timer<E, S, L, R, H, G> {
                Timer {
                    callbacks: BinaryHeap::new(),
                    time: 0,
                    id: 0
                }
            }

            pub fn update(
                &mut self,
                dt: u64

            ) -> Vec<Box<FnMut(&mut H, Handle<E, S, L, R, H, G>)>> {

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
                f: Box<FnMut(&mut H, Handle<E, S, L, R, H, G>)>,
                time: u64

            ) -> u32 {
                self.id += 1;
                self.callbacks.push(TimerCallback {
                    func: f,
                    time: self.time + time,
                    id: self.id
                });
                self.id
            }

            pub fn cancel(&mut self, _: u32) {
                // TODO push into cancel list
            }

        }


        // Timer Callback Wrapper ---------------------------------------------
        struct TimerCallback<
            E: $e,
            S: $s,
            L: $b<S>,
            R: $r,
            H: $h<E, S, L, R, G>,
            G: $g<S, L, R>
        > {
            func: Box<FnMut(&mut H, Handle<E, S, L, R, H, G>)>,
            time: u64,
            id: u32
        }

        impl<
            E: $e,
            S: $s,
            L: $b<S>,
            R: $r,
            H: $h<E, S, L, R, G>,
            G: $g<S, L, R>

        > Eq for TimerCallback<E, S, L, R, H, G> {}

        impl<
            E: $e,
            S: $s,
            L: $b<S>,
            R: $r,
            H: $h<E, S, L, R, G>,
            G: $g<S, L, R>

        > PartialEq for TimerCallback<E, S, L, R, H, G> {
            fn eq(&self, other: &TimerCallback<E, S, L, R, H, G>) -> bool {
                self.id == other.id
            }
        }

        impl<
            E: $e,
            S: $s,
            L: $b<S>,
            R: $r,
            H: $h<E, S, L, R, G>,
            G: $g<S, L, R>

        > Ord for TimerCallback<E, S, L, R, H, G> {
            // Explicitly implement the trait so the queue becomes a min-heap
            // instead of a max-heap.
            fn cmp(&self, other: &TimerCallback<E, S, L, R, H, G>) -> cmp::Ordering {
                other.time.cmp(&self.time)
            }
        }

        impl<
            E: $e,
            S: $s,
            L: $b<S>,
            R: $r,
            H: $h<E, S, L, R, G>,
            G: $g<S, L, R>

        > PartialOrd for TimerCallback<E, S, L, R, H, G> {
            fn partial_cmp(
                &self, other: &TimerCallback<E, S, L, R, H, G>

            ) -> Option<cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

    }
}

