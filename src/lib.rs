extern crate num_traits;
use num_traits::Float;

pub trait Sample<T> {
    const MAX_I32: T;
    const MAX_I24: T;
    const MAX_I16: T;

    fn to_s32_le(&self) -> ([u8; 4], bool);
    fn to_s24_3_le(&self) -> ([u8; 3], bool);
    fn to_s24_4_le(&self) -> ([u8; 4], bool);
    fn to_s16_le(&self) -> ([u8; 2], bool);
    fn to_f64_le(&self) -> ([u8; 8], bool);
    fn to_f32_le(&self) -> ([u8; 4], bool);
}


fn clamp_int_f64(invalue: f64, maxval: f64) -> (f64, bool) {
    let mut val = invalue;
    let mut clipped = false;
    if val >= maxval {
        clipped = true;
        val = maxval - 1.0;
    }
    else if val < -maxval {
        clipped = true;
        val = -maxval
    }
    (val, clipped)
}

fn clamp_int<T: Float>(invalue: T, maxval: T) -> (T, bool) {
    let mut val = invalue;
    let mut clipped = false;
    if val >= maxval {
        clipped = true;
        val = maxval - T::from(1.0).unwrap();
    }
    else if val < -maxval {
        clipped = true;
        val = -maxval
    }
    (val, clipped)
}

fn clamp_float_f64(invalue: f64) -> (f64, bool) {
    let mut val = invalue;
    let mut clipped = false;
    if val >= 1.0 {
        clipped = true;
        val = 1.0;
    }
    else if val < -1.0 {
        clipped = true;
        val = -1.0
    }
    (val, clipped)
}

impl Sample<f64> for f64 {
    const MAX_I32: f64 = 2147483648.0;
    const MAX_I24: f64 = 8388608.0;
    const MAX_I16: f64 = 32768.0;


    fn to_s16_le(&self) -> ([u8; 2], bool) {
        let val = self * f64::MAX_I16;
        let (val, clipped) = clamp_int_f64(val, f64::MAX_I16); 
        ((val as i16).to_le_bytes(), clipped)
    } 

    fn to_s32_le(&self) -> ([u8; 4], bool) {
        let val = self * f64::MAX_I32;
        let (val, clipped) = clamp_int_f64(val, f64::MAX_I32); 
        ((val as i32).to_le_bytes(), clipped)
    } 

    fn to_s24_3_le(&self) -> ([u8; 3], bool) {
        let val = self * f64::MAX_I32;
        let (val, clipped) = clamp_int_f64(val, f64::MAX_I32);
        let bytes = (val as i32).to_le_bytes();
        ([bytes[1], bytes[2], bytes[3]], clipped)
    } 

    fn to_s24_4_le(&self) -> ([u8; 4], bool) {
        let val = self * f64::MAX_I32;
        let (val, clipped) = clamp_int_f64(val, f64::MAX_I32);
        let bytes = (val as i32).to_le_bytes();
        ([bytes[1], bytes[2], bytes[3], 0], clipped)
    } 

    fn to_f64_le(&self) -> ([u8; 8], bool) {
        let mut clipped = false; 
        let mut val = *self;
        if val > 1.0 {
            clipped = true;
            val = 1.0;
        }
        else if val < -1.0 {
            clipped = true;
            val = -1.0;
        }
        (val.to_le_bytes(), clipped)
    } 

    fn to_f32_le(&self) -> ([u8; 4], bool) {
        let mut clipped = false; 
        let mut val = *self as f32;
        if val > 1.0 {
            clipped = true;
            val = 1.0;
        }
        else if val < -1.0 {
            clipped = true;
            val = -1.0;
        }
        (val.to_le_bytes(), clipped)
    } 

}

#[cfg(test)]
mod tests {
    use crate::Sample;
    #[test]
    fn check_f64_to_s32le() {
        let val: f64 = 0.256789;
        assert_eq!(val.to_s32_le(), ([66, 118, 222, 32], false));
        let val: f64 = -0.256789;
        assert_eq!(val.to_s32_le(), ([190, 137, 33, 223], false));
        let val: f64 = 1.1;
        assert_eq!(val.to_s32_le(), ([255, 255, 255, 127], true));
        let val: f64 = -1.1;
        assert_eq!(val.to_s32_le(), ([0, 0, 0, 128], true));
    }

    #[test]
    fn check_f64_to_s243le() {
        let val: f64 = 0.256789;
        assert_eq!(val.to_s24_3_le(), ([118, 222, 32], false));
        let val: f64 = -0.256789;
        assert_eq!(val.to_s24_3_le(), ([137, 33, 223], false));
        let val: f64 = 1.1;
        assert_eq!(val.to_s24_3_le(), ([255, 255, 127], true));
        let val: f64 = -1.1;
        assert_eq!(val.to_s24_3_le(), ([0, 0, 128], true));
    }

    #[test]
    fn check_f64_to_s244le() {
        let val: f64 = 0.256789;
        assert_eq!(val.to_s24_4_le(), ([118, 222, 32, 0], false));
        let val: f64 = -0.256789;
        assert_eq!(val.to_s24_4_le(), ([137, 33, 223, 0], false));
        let val: f64 = 1.1;
        assert_eq!(val.to_s24_4_le(), ([255, 255, 127, 0], true));
        let val: f64 = -1.1;
        assert_eq!(val.to_s24_4_le(), ([0, 0, 128, 0], true));
    }

    #[test]
    fn check_f64_to_s16le() {
        let val: f64 = 0.256789;
        assert_eq!(val.to_s16_le(), ([222, 32], false));
        let val: f64 = -0.256789;
        assert_eq!(val.to_s16_le(), ([34, 223], false));
    }

    #[test]
    fn check_f64_to_f32le() {
        let val: f64 = 0.256789;
        let exp = (0.256789 as f32).to_le_bytes();
        assert_eq!(val.to_f32_le(), (exp, false));
        let val: f64 = -0.256789;
        let exp = (-0.256789 as f32).to_le_bytes();
        assert_eq!(val.to_f32_le(), (exp, false));
        let val: f64 = 1.1;
        let exp = (1.0 as f32).to_le_bytes();
        assert_eq!(val.to_f32_le(), (exp, true));
        let val: f64 = -1.1;
        let exp = (-1.0 as f32).to_le_bytes();
        assert_eq!(val.to_f32_le(), (exp, true));
    }

    #[test]
    fn check_f64_to_f64le() {
        let val: f64 = 0.256789;
        let exp = (0.256789 as f64).to_le_bytes();
        assert_eq!(val.to_f64_le(), (exp, false));
        let val: f64 = -0.256789;
        let exp = (-0.256789 as f64).to_le_bytes();
        assert_eq!(val.to_f64_le(), (exp, false));
        let val: f64 = 1.1;
        let exp = (1.0 as f64).to_le_bytes();
        assert_eq!(val.to_f64_le(), (exp, true));
        let val: f64 = -1.1;
        let exp = (-1.0 as f64).to_le_bytes();
        assert_eq!(val.to_f64_le(), (exp, true));
    }
}
