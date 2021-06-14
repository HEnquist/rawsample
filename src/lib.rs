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

    fn from_s32_le(bytes: [u8; 4]) -> Self;
    fn from_s16_le(bytes: [u8; 2]) -> Self;
    fn from_s24_3_le(bytes: [u8; 3]) -> Self;
    fn from_s24_4_le(bytes: [u8; 4]) -> Self;
    fn from_f32_le(bytes: [u8; 4]) -> Self;
    fn from_f64_le(bytes: [u8; 8]) -> Self;
}


fn clamp_int<T: Float>(invalue: T, maxval: T) -> (T, bool) {
    let mut val = invalue;
    let mut clipped = false;
    if val >= maxval {
        clipped = true;
        val = maxval - T::one();
    }
    else if val < -maxval {
        clipped = true;
        val = -maxval
    }
    (val, clipped)
}

fn clamp_float<T: Float>(invalue: T) -> (T, bool) {
    let mut val = invalue;
    let mut clipped = false;
    if val >= T::one() {
        clipped = true;
        val = T::one();
    }
    else if val < -T::one() {
        clipped = true;
        val = -T::one()
    }
    (val, clipped)
}

impl Sample<f64> for f64 {
    const MAX_I32: f64 = 2147483648.0;
    const MAX_I24: f64 = 8388608.0;
    const MAX_I16: f64 = 32768.0;


    fn to_s16_le(&self) -> ([u8; 2], bool) {
        let val = self * f64::MAX_I16;
        let (val, clipped) = clamp_int(val, f64::MAX_I16); 
        ((val as i16).to_le_bytes(), clipped)
    } 

    fn to_s32_le(&self) -> ([u8; 4], bool) {
        let val = self * f64::MAX_I32;
        let (val, clipped) = clamp_int(val, f64::MAX_I32); 
        ((val as i32).to_le_bytes(), clipped)
    } 

    fn to_s24_3_le(&self) -> ([u8; 3], bool) {
        let val = self * f64::MAX_I32;
        let (val, clipped) = clamp_int(val, f64::MAX_I32);
        let bytes = (val as i32).to_le_bytes();
        ([bytes[1], bytes[2], bytes[3]], clipped)
    } 

    fn to_s24_4_le(&self) -> ([u8; 4], bool) {
        let val = self * f64::MAX_I32;
        let (val, clipped) = clamp_int(val, f64::MAX_I32);
        let bytes = (val as i32).to_le_bytes();
        ([bytes[1], bytes[2], bytes[3], 0], clipped)
    } 

    fn to_f64_le(&self) -> ([u8; 8], bool) {
        let val = *self;
        let (val, clipped) = clamp_float(val);
        (val.to_le_bytes(), clipped)
    } 

    fn to_f32_le(&self) -> ([u8; 4], bool) {
        let val = *self as f32;
        let (val, clipped) = clamp_float(val);
        (val.to_le_bytes(), clipped)
    } 

    fn from_s32_le(bytes: [u8; 4]) -> Self {
        let intvalue = i32::from_le_bytes(bytes);
        intvalue as f64 / f64::MAX_I32
    }

    fn from_s16_le(bytes: [u8; 2]) -> Self {
        let intvalue = i16::from_le_bytes(bytes);
        intvalue as f64 / f64::MAX_I16
    }

    fn from_s24_3_le(bytes: [u8; 3]) -> Self {
        let padded = [0, bytes[0], bytes[1], bytes[2]];
        let intvalue = i32::from_le_bytes(padded);
        intvalue as f64 / f64::MAX_I32
    }

    fn from_s24_4_le(bytes: [u8; 4]) -> Self {
        let padded = [0, bytes[0], bytes[1], bytes[2]];
        let intvalue = i32::from_le_bytes(padded);
        intvalue as f64 / f64::MAX_I32
    }

    fn from_f32_le(bytes: [u8; 4]) -> Self {
        f32::from_le_bytes(bytes) as f64
    }

    fn from_f64_le(bytes: [u8; 8]) -> Self {
        f64::from_le_bytes(bytes)
    }
}


impl Sample<f32> for f32 {
    const MAX_I32: f32 = 2147483648.0;
    const MAX_I24: f32 = 8388608.0;
    const MAX_I16: f32 = 32768.0;


    fn to_s16_le(&self) -> ([u8; 2], bool) {
        let val = self * f32::MAX_I16;
        let (val, clipped) = clamp_int(val, f32::MAX_I16); 
        ((val as i16).to_le_bytes(), clipped)
    } 

    fn to_s32_le(&self) -> ([u8; 4], bool) {
        let val = self * f32::MAX_I32;
        let (val, clipped) = clamp_int(val, f32::MAX_I32); 
        ((val as i32).to_le_bytes(), clipped)
    } 

    fn to_s24_3_le(&self) -> ([u8; 3], bool) {
        let val = self * f32::MAX_I32;
        let (val, clipped) = clamp_int(val, f32::MAX_I32);
        let bytes = (val as i32).to_le_bytes();
        ([bytes[1], bytes[2], bytes[3]], clipped)
    } 

    fn to_s24_4_le(&self) -> ([u8; 4], bool) {
        let val = self * f32::MAX_I32;
        let (val, clipped) = clamp_int(val, f32::MAX_I32);
        let bytes = (val as i32).to_le_bytes();
        ([bytes[1], bytes[2], bytes[3], 0], clipped)
    } 

    fn to_f64_le(&self) -> ([u8; 8], bool) {
        let val = *self as f64;
        let (val, clipped) = clamp_float(val);
        (val.to_le_bytes(), clipped)
    } 

    fn to_f32_le(&self) -> ([u8; 4], bool) {
        let (val, clipped) = clamp_float(*self);
        (val.to_le_bytes(), clipped)
    } 

    fn from_s32_le(bytes: [u8; 4]) -> Self {
        let intvalue = i32::from_le_bytes(bytes);
        intvalue as f32 / f32::MAX_I32
    }

    fn from_s16_le(bytes: [u8; 2]) -> Self {
        let intvalue = i16::from_le_bytes(bytes);
        intvalue as f32 / f32::MAX_I16
    }

    fn from_s24_3_le(bytes: [u8; 3]) -> Self {
        let padded = [0, bytes[0], bytes[1], bytes[2]];
        let intvalue = i32::from_le_bytes(padded);
        intvalue as f32 / f32::MAX_I32
    }

    fn from_s24_4_le(bytes: [u8; 4]) -> Self {
        let padded = [0, bytes[0], bytes[1], bytes[2]];
        let intvalue = i32::from_le_bytes(padded);
        intvalue as f32 / f32::MAX_I32
    }

    fn from_f32_le(bytes: [u8; 4]) -> Self {
        f32::from_le_bytes(bytes)
    }

    fn from_f64_le(bytes: [u8; 8]) -> Self {
        f64::from_le_bytes(bytes) as f32 
    }
}

#[cfg(test)]
mod tests {
    use crate::Sample;

    // -----------------
    //       f64 
    // -----------------

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
    fn check_f64_from_s32le() {
        let data = [0, 0, 64, 32];
        assert_eq!(f64::from_s32_le(data), 0.251953125);
        let data = [0, 0, 64, 223];
        assert_eq!(f64::from_s32_le(data), -0.255859375);
        let data = [0, 0, 0, 128];
        assert_eq!(f64::from_s32_le(data), -1.0);
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
    fn check_f64_from_s243le() {
        let data = [0, 64, 32];
        assert_eq!(f64::from_s24_3_le(data), 0.251953125);
        let data = [0, 64, 223];
        assert_eq!(f64::from_s24_3_le(data), -0.255859375);
        let data = [0, 0, 128];
        assert_eq!(f64::from_s24_3_le(data), -1.0);
    }

    #[test]
    fn check_f64_from_s244le() {
        let data = [0, 64, 32, 0];
        assert_eq!(f64::from_s24_4_le(data), 0.251953125);
        let data = [0, 64, 223, 0];
        assert_eq!(f64::from_s24_4_le(data), -0.255859375);
        let data = [0, 0, 128, 0];
        assert_eq!(f64::from_s24_4_le(data), -1.0);
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

    // -----------------
    //        f32
    // -----------------
    #[test]
    fn check_f32_to_s32le() {
        let val: f32 = 0.256789;
        assert_eq!(val.to_s32_le(), ([64, 118, 222, 32], false));
        let val: f32 = -0.256789;
        assert_eq!(val.to_s32_le(), ([192, 137, 33, 223], false));
        let val: f32 = 1.1;
        assert_eq!(val.to_s32_le(), ([255, 255, 255, 127], true));
        let val: f32 = -1.1;
        assert_eq!(val.to_s32_le(), ([0, 0, 0, 128], true));
    }

    #[test]
    fn check_f32_from_s32le() {
        let data = [0, 0, 64, 32];
        assert_eq!(f32::from_s32_le(data), 0.251953125);
        let data = [0, 0, 64, 223];
        assert_eq!(f32::from_s32_le(data), -0.255859375);
        let data = [0, 0, 0, 128];
        assert_eq!(f32::from_s32_le(data), -1.0);
    }

    #[test]
    fn check_f32_to_s243le() {
        let val: f32 = 0.256789;
        assert_eq!(val.to_s24_3_le(), ([118, 222, 32], false));
        let val: f32 = -0.256789;
        assert_eq!(val.to_s24_3_le(), ([137, 33, 223], false));
        let val: f32 = 1.1;
        assert_eq!(val.to_s24_3_le(), ([255, 255, 127], true));
        let val: f32 = -1.1;
        assert_eq!(val.to_s24_3_le(), ([0, 0, 128], true));
    }

    #[test]
    fn check_f32_from_s243le() {
        let data = [0, 64, 32];
        assert_eq!(f32::from_s24_3_le(data), 0.251953125);
        let data = [0, 64, 223];
        assert_eq!(f32::from_s24_3_le(data), -0.255859375);
        let data = [0, 0, 128];
        assert_eq!(f32::from_s24_3_le(data), -1.0);
    }

    #[test]
    fn check_f32_from_s244le() {
        let data = [0, 64, 32, 0];
        assert_eq!(f32::from_s24_4_le(data), 0.251953125);
        let data = [0, 64, 223, 0];
        assert_eq!(f32::from_s24_4_le(data), -0.255859375);
        let data = [0, 0, 128, 0];
        assert_eq!(f32::from_s24_4_le(data), -1.0);
    }

    #[test]
    fn check_f32_to_s244le() {
        let val: f32 = 0.256789;
        assert_eq!(val.to_s24_4_le(), ([118, 222, 32, 0], false));
        let val: f32 = -0.256789;
        assert_eq!(val.to_s24_4_le(), ([137, 33, 223, 0], false));
        let val: f32 = 1.1;
        assert_eq!(val.to_s24_4_le(), ([255, 255, 127, 0], true));
        let val: f32 = -1.1;
        assert_eq!(val.to_s24_4_le(), ([0, 0, 128, 0], true));
    }

    #[test]
    fn check_f32_to_s16le() {
        let val: f32 = 0.256789;
        assert_eq!(val.to_s16_le(), ([222, 32], false));
        let val: f32 = -0.256789;
        assert_eq!(val.to_s16_le(), ([34, 223], false));
    }

    #[test]
    fn check_f32_to_f32le() {
        let val: f32 = 0.256789;
        let exp = (0.256789 as f32).to_le_bytes();
        assert_eq!(val.to_f32_le(), (exp, false));
        let val: f32 = -0.256789;
        let exp = (-0.256789 as f32).to_le_bytes();
        assert_eq!(val.to_f32_le(), (exp, false));
        let val: f32 = 1.1;
        let exp = (1.0 as f32).to_le_bytes();
        assert_eq!(val.to_f32_le(), (exp, true));
        let val: f32 = -1.1;
        let exp = (-1.0 as f32).to_le_bytes();
        assert_eq!(val.to_f32_le(), (exp, true));
    }

    #[test]
    fn check_f32_to_f64le() {
        let val: f32 = 0.256789;
        let exp = ((0.256789 as f32) as f64).to_le_bytes();
        assert_eq!(val.to_f64_le(), (exp, false));
        let val: f32 = -0.256789;
        let exp = ((-0.256789 as f32) as f64).to_le_bytes();
        assert_eq!(val.to_f64_le(), (exp, false));
        let val: f32 = 1.1;
        let exp = (1.0 as f64).to_le_bytes();
        assert_eq!(val.to_f64_le(), (exp, true));
        let val: f32 = -1.1;
        let exp = (-1.0 as f64).to_le_bytes();
        assert_eq!(val.to_f64_le(), (exp, true));
    }

    #[test]
    fn dummy() {
        let v1: f32 = 127.0;
        let v2: f32 = 128.0;
        let v3: f32 = 129.0;

        println!("{}", v1 as i8);
        println!("{}", v2 as i8);
        println!("{}", v3 as i8);

        let v1: f32 = 255.0;
        let v2: f32 = 256.0;
        let v3: f32 = 257.0;

        println!("{}", v1 as u8);
        println!("{}", v2 as u8);
        println!("{}", v3 as u8);
        assert!(false);
    }
}
