fn ifloor_div(a: i64, b: i64) -> i64 {
    if a > 0 && b > 0 || a < 0 && b < 0 || a % b == 0 {
        a / b
    } else {
        a / b - 1
    }
}

fn ffloor_div(a: f64, b: f64) -> f64 {
    (a / b).floor()
}

fn imod(a: i64, b: i64) -> i64 {
    a - ifloor_div(a, b) * b
}

fn fmod(a: f64, b: f64) -> f64 {
    if a > 0.0 && is_positive_infinite(b) || a < 0.0 && is_negative_infinite(b) {
        a
    } else if a > 0.0 && is_negative_infinite(b) || a < 0.0 && is_positive_infinite(b) {
        b
    } else {
        a - (a / b).floor() * b
    }
}

fn is_positive_infinite(n: f64) -> bool {
    n.is_infinite() && n.is_sign_positive()
}

fn is_negative_infinite(n: f64) -> bool {
    n.is_infinite() && n.is_sign_negative()
}

fn shift_left(a: i64, n: i64) -> i64 {
    if n >= 64 {
        0
    } else if n >= 0 {
        a << n
    } else {
        shift_right(a, -n)
    }
}

fn shift_right(a: i64, n: i64) -> i64 {
    if n >= 64 {
        0
    } else if n >= 0 {
        (a as u64 >> n) as i64
    } else {
        return shift_left(a, -n);
    }
}

pub fn float_to_integer(n: f64) -> Option<i64> {
    let i = n as i64;
    if i as f64 == n {
        Some(i)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modulo() {
        assert_eq!(imod(5, 3), 2);
        assert_eq!(imod(-5, 3), 1);
        assert_eq!(imod(-2, 3), 1);
        assert_eq!(imod(2, -3), -1);
        assert_eq!(fmod(5.0, -3.0), -1.0);
        assert_eq!(fmod(-5.0, -3.0), -2.0);
    }

    #[test]
    fn floor_div() {
        assert_eq!(ifloor_div(5, 3), 1);
        assert_eq!(ifloor_div(-5, 3), -2);
        assert_eq!(ffloor_div(5.0, -3.0), -2.0);
        assert_eq!(ffloor_div(-5.0, -3.0), 1.0);
    }

    #[test]
    fn bit_shift() {
        assert_eq!(shift_left(2, -1), 1);
        assert_eq!(shift_left(1, 1), 2);
        assert_eq!(shift_right(-1, 63), 1);
        assert_eq!(shift_right(1, 64), 0);
        assert_eq!(shift_left(0xFF, 100), 0);
        assert_eq!(shift_left(0xFF, -4), 0x0F);
        assert_eq!(shift_right(0xFF, 100), 0);
        assert_eq!(shift_right(0xFF, -4), 0xFF0);
    }

    #[test]
    fn float2integer() {
        assert_eq!(float_to_integer(99.0), Some(99));
        assert_eq!(float_to_integer(99.9), None);
        assert_eq!(float_to_integer(-99.0), Some(-99));
        assert_eq!(float_to_integer(-99.9), None);
    }
}
