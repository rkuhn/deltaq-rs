use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq)]
pub struct CDF {
    data: Vec<u16>,
    bin_size: f64,
}

impl CDF {
    pub fn new(data: Vec<f64>, bin_size: f64) -> Result<Self, &'static str> {
        if !data.iter().all(|&x| x >= 0.0 && x <= 1.0) {
            return Err("Data vector must contain values between 0 and 1");
        }
        if !data.windows(2).all(|w| w[0] <= w[1]) {
            return Err("Data vector must contain monotonically increasing values");
        }
        let converted_data: Vec<u16> = data.iter().map(|&x| (x * 65535.0) as u16).collect();
        Ok(Self {
            data: converted_data,
            bin_size,
        })
    }

    pub fn choice(&self, fraction: f64, other: &CDF) -> Result<CDF, &'static str> {
        if self.bin_size != other.bin_size {
            return Err("CDFs must have the same bin size for addition");
        }
        if self.data.len() != other.data.len() {
            return Err("CDFs must have the same length for addition");
        }
        if fraction < 0.0 || fraction > 1.0 {
            return Err("Fraction must be between 0 and 1");
        }
        let my_fraction = to_int(fraction);
        let fraction = to_int(1.0 - fraction);
        let combined_data: Vec<u16> = self
            .data
            .iter()
            .zip(&other.data)
            .map(|(&x, &y)| {
                mul(x, my_fraction)
                    .checked_add(mul(y, fraction))
                    .expect("addition overflow")
            })
            .collect();
        Ok(CDF {
            data: combined_data,
            bin_size: self.bin_size,
        })
    }

    pub fn for_all(&self, other: &CDF) -> Result<CDF, &'static str> {
        if self.bin_size != other.bin_size {
            return Err("CDFs must have the same bin size for for_all");
        }
        if self.data.len() != other.data.len() {
            return Err("CDFs must have the same length for for_all");
        }
        let multiplied_data: Vec<u16> = self
            .data
            .iter()
            .zip(&other.data)
            .map(|(&x, &y)| mul(x, y))
            .collect();
        Ok(CDF {
            data: multiplied_data,
            bin_size: self.bin_size,
        })
    }

    pub fn for_some(&self, other: &CDF) -> Result<CDF, &'static str> {
        if self.bin_size != other.bin_size {
            return Err("CDFs must have the same bin size for for_some");
        }
        if self.data.len() != other.data.len() {
            return Err("CDFs must have the same length for for_some");
        }
        let multiplied_data: Vec<u16> = self
            .data
            .iter()
            .zip(&other.data)
            .map(|(&x, &y)| {
                u16::try_from(
                    (x as u32 + y as u32)
                        .checked_sub(mul(x, y) as u32)
                        .expect("subtraction underflow during for_some"),
                )
                .expect("overflow during for_some")
            })
            .collect();
        Ok(CDF {
            data: multiplied_data,
            bin_size: self.bin_size,
        })
    }

    pub fn convolve(&self, other: &CDF) -> Result<CDF, &'static str> {
        if self.bin_size != other.bin_size {
            return Err("CDFs must have the same bin size for convolution");
        }
        if self.data.len() != other.data.len() {
            return Err("CDFs must have the same length for convolution");
        }
        let len = self.data.len();
        let mut convolved_data: Vec<u16> = vec![0; len];
        for i in 0..len {
            for j in 0..len - i {
                let other = if j == 0 {
                    other.data[j]
                } else {
                    other.data[j] - other.data[j - 1]
                };
                convolved_data[i + j] += mul(self.data[i], other);
            }
        }
        Ok(CDF {
            data: convolved_data,
            bin_size: self.bin_size,
        })
    }
}

impl PartialOrd for CDF {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.bin_size != other.bin_size {
            return None;
        }
        let mut ret = None;
        for (l, r) in self.data.iter().zip(&other.data) {
            if l < r {
                if ret == Some(Ordering::Greater) {
                    return None;
                }
                ret = Some(Ordering::Less);
            } else if l > r {
                if ret == Some(Ordering::Less) {
                    return None;
                }
                ret = Some(Ordering::Greater);
            }
        }
        ret.or(Some(Ordering::Equal))
    }
}

fn mul(x: u16, y: u16) -> u16 {
    ((x as u32 * y as u32 + 65535) >> 16) as u16
}

fn to_int(x: f64) -> u16 {
    (x * 65536.0).min(65535.0) as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let data = vec![0.0, 0.25, 0.5, 0.75, 1.0];
        let cdf = CDF::new(data.clone(), 0.25).unwrap();
        assert_eq!(cdf.data, vec![0, 16383, 32767, 49151, 65535]);
        assert_eq!(cdf.bin_size, 0.25);

        let data = vec![0.0, 0.25, 0.5, 0.75, 1.1];
        let cdf = CDF::new(data.clone(), 0.25);
        assert_eq!(cdf, Err("Data vector must contain values between 0 and 1"));

        let data = vec![0.0, 0.25, 0.5, 0.75, 0.5];
        let cdf = CDF::new(data.clone(), 0.25);
        assert_eq!(
            cdf,
            Err("Data vector must contain monotonically increasing values")
        );
    }

    #[test]
    fn test_choice() {
        let left = CDF::new(vec![0.0, 0.0, 0.5, 1.0, 1.0], 0.25).unwrap();
        let right = CDF::new(vec![0.0, 1.0, 1.0, 1.0, 1.0], 0.25).unwrap();
        let added = left.choice(0.7, &right).unwrap();
        assert_eq!(
            added,
            CDF::new(vec![0.0, 0.3, 0.65, 1.0, 1.0], 0.25).unwrap()
        );
        let added = left.choice(1.0, &right).unwrap();
        assert_eq!(
            added,
            CDF::new(vec![0.0, 0.0, 0.5, 1.0, 1.0], 0.25).unwrap()
        );
    }

    #[test]
    fn test_convolve_step() {
        let left = CDF::new(vec![0.0, 1.0, 1.0, 1.0, 1.0], 1.0).unwrap();
        let right = CDF::new(vec![0.0, 0.0, 1.0, 1.0, 1.0], 1.0).unwrap();
        let convolved = left.convolve(&right).unwrap();
        assert_eq!(
            convolved,
            CDF::new(vec![0.0, 0.0, 0.0, 1.0, 1.0], 1.0).unwrap()
        );
    }

    #[test]
    fn test_convolve_two() {
        let left = CDF::new(vec![0.0, 0.3, 0.3, 1.0, 1.0, 1.0, 1.0], 1.0).unwrap();
        let right = CDF::new(vec![0.0, 0.0, 0.6, 1.0, 1.0, 1.0, 1.0], 1.0).unwrap();
        let convolved = left.convolve(&right).unwrap();
        assert_eq!(
            convolved,
            CDF::new(vec![0.0, 0.0, 0.0, 0.18, 0.3, 0.72, 1.0], 1.0).unwrap()
        );
    }

    #[test]
    fn test_for_all() {
        let left = CDF::new(vec![0.0, 0.5, 0.75, 1.0], 0.25).unwrap();
        let right = CDF::new(vec![0.0, 0.25, 0.5, 1.0], 0.25).unwrap();
        let result = left.for_all(&right).unwrap();
        assert_eq!(
            result,
            CDF::new(vec![0.0, 0.12501, 0.375, 1.0], 0.25).unwrap()
        );
    }

    #[test]
    fn test_for_some() {
        let left = CDF::new(vec![0.0, 0.5, 0.75, 1.0], 0.25).unwrap();
        let right = CDF::new(vec![0.0, 0.25, 0.5, 1.0], 0.25).unwrap();
        let result = left.for_some(&right).unwrap();
        assert_eq!(
            result,
            CDF::new(vec![0.0, 0.62499, 0.875, 1.0], 0.25).unwrap()
        );
    }

    #[test]
    fn partial_ord() {
        let left = CDF::new(vec![0.0, 0.3, 0.3, 1.0], 1.0).unwrap();
        let right = CDF::new(vec![0.0, 0.0, 0.6, 1.0], 1.0).unwrap();
        let top = CDF::new(vec![0.0, 0.3, 0.6, 1.0], 1.0).unwrap();
        let bottom = CDF::new(vec![0.0, 0.0, 0.3, 1.0], 1.0).unwrap();
        assert_ne!(left, right);
        assert!(!(left < right));
        assert!(!(right > left));
        assert!(!(left <= right));
        assert!(!(right >= left));
        assert!(left < top);
        assert!(top > left);
        assert!(left <= top);
        assert!(top >= left);
        assert!(right < top);
        assert!(top > right);
        assert!(right <= top);
        assert!(top >= right);
        assert!(left > bottom);
        assert!(bottom < left);
        assert!(left >= bottom);
        assert!(bottom <= left);
        assert!(right > bottom);
        assert!(bottom < right);
        assert!(right >= bottom);
        assert!(bottom <= right);
    }
}
