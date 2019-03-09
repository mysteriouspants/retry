use std::time::Duration;
use std::thread::sleep;

/// An exponential backoff, measured in milliseconds, which
/// retries until it reaches `max_retries`. As an exponential
/// backoff, it follows the formula *an^b+c*, where *a* is
/// `coefficient`, *b* is `exponent`, *c* is `constant`, and
/// *n* is the attempt number. This means that the total time
/// possible to spend waiting in a retry is given by the sum,
/// from *1* to `max_retries`, of *an^b+c*.
pub struct ExponentialBackoff<T, E> {

    /// Block describing whether a given `Result` ought to be
    /// considered retriable.
    pub should_retry: Box<Fn(&Result<T, E>) -> bool + Send + Sync>,

    /// The maximum number of times to retry the operation
    /// before giving up.
    pub max_retries: u8,

    /// The constant to add to each backoff time.
    pub constant: f32,

    /// The coefficient to multiply each exponentiated backoff
    /// time by, before adding `constant`.
    pub coefficient: f32,

    /// The exponent to raise the retry attempt to.
    pub exponent: f32
}

impl <T, E> ExponentialBackoff<T, E> {
    
    /// A default backoff configured for networking with a
    /// [61-second total backoff time](https://www.wolframalpha.com/input/?i=sum+0%2B1000t%5E1.5+from+1+to+7).
    pub fn new_with_defaults<
        TShouldRetry: Fn(&Result<T, E>) -> bool + Send + Sync + 'static
    > (should_retry: TShouldRetry) -> ExponentialBackoff<T, E> {
        // https://www.wolframalpha.com/input/?i=sum+0%2B1000t%5E1.5+from+1+to+7
        return ExponentialBackoff::new(7, 0.0, 1000.0, 0.5, should_retry);
    }

    /// Creates a new backoff.
    pub fn new<
        TShouldRetry: Fn(&Result<T, E>) -> bool + Send + Sync + 'static
    > (
        max_retries: u8,
        constant: f32,
        coefficient: f32,
        exponent: f32,
        should_retry: TShouldRetry
    ) -> ExponentialBackoff<T, E> {
        return ExponentialBackoff {
            should_retry: Box::new(should_retry),
            max_retries: max_retries,
            constant: constant,
            coefficient: coefficient,
            exponent: exponent
        };
    }

    /// Executes an operation, retrying it until it succeeds
    /// or the maximum number of retries has been exhausted.
    pub fn retry<TRetriable>(
        &self,
        mut retriable_block: TRetriable
    ) -> Result<T, E> where TRetriable : FnMut() -> Result<T, E> {
        let mut retry_count: u8 = 0;

        loop {
            retry_count += 1;
            let result = retriable_block();

            if retry_count == self.max_retries
                || !(self.should_retry)(&result) {
                return result;
            } else {
                let backoff_time = self.constant + self.coefficient
                    * (retry_count as f32).powf(self.exponent);
                sleep(Duration::from_millis(backoff_time as u64));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ExponentialBackoff;

    #[test]
    fn succeeds_after_two_retries() {
        let result = retry_until_true(
            vec![false, false, true]
        );

        match result {
            Ok(_) => assert!(true),
            Err(_) => assert!(false)
        };
    }

    #[test]
    fn fails_after_exhausting_retries() {
        let v = vec![
            false, false, false, false, false,
            false, false, false
        ];

        let result = retry_until_true(v);

        match result {
            Ok(_) => assert!(false), // succeeded? impossible!
            Err(_) => assert!(true) // failed as expected
        };
    }

    fn retry_until_true(
        mut v: Vec<bool>
    ) -> Result<bool, bool> {
        let backoff = ExponentialBackoff::new(
            // tighten the timings to make the tests run faster
            7, 0.0, 1.0, 2.0,
            // retry until there is no "error"
            |result: &Result<bool, bool>| !result.is_ok()
        );

        let result = backoff.retry(|| {
            return match v.pop() {
                Some(true) => Ok(true),
                Some(false) => Err(false),
                None => Err(false)
            };
        });

        return result;
    }
}

