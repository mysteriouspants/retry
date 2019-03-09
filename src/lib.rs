use std::time::Duration;
use std::thread::sleep;

pub struct ExponentialBackoff<T, E> {
    pub should_retry: Box<Fn(&Result<T, E>) -> bool + Send + Sync>,

    pub max_retries: u8,

    pub constant: f32,
    pub coefficient: f32,
    pub exponent: f32
}

impl <T, E> ExponentialBackoff<T, E> {
    
    pub fn new_with_defaults<
        TShouldRetry: Fn(&Result<T, E>) -> bool + Send + Sync + 'static
    > (should_retry: TShouldRetry) -> ExponentialBackoff<T, E> {
        // https://www.wolframalpha.com/input/?i=sum+0%2B1000t%5E1.5+from+1+to+7
        return ExponentialBackoff::new(7, 0.0, 1000.0, 0.5, should_retry);
    }

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
