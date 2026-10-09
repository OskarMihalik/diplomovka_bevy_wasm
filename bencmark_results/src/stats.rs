//! Summary statistics over the runs of one configuration.

use statrs::distribution::{ContinuousCDF, StudentsT};

#[derive(Clone, Copy, Debug)]
pub struct Summary {
    pub n: usize,
    pub mean: f64,
    /// sample standard deviation (n - 1), 0 for a single run
    pub sd: f64,
}

impl Summary {
    pub fn of(values: &[f64]) -> Option<Self> {
        let n = values.len();
        if n == 0 {
            return None;
        }
        let mean = values.iter().sum::<f64>() / n as f64;
        let sd = if n > 1 {
            (values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (n - 1) as f64).sqrt()
        } else {
            0.
        };
        Some(Self { n, mean, sd })
    }
}

#[allow(dead_code)] // t, df and g aren't in the paper tables, kept for checking
pub struct Welch {
    /// (a - b) / b in %
    pub diff_percent: f64,
    pub t: f64,
    pub df: f64,
    /// two-sided
    pub p: f64,
    /// Hedges' g, effect size corrected for small samples
    pub g: f64,
}

/// Welch's t-test, doesn't assume equal variances. None with fewer than 2 runs on either side.
pub fn welch(a: Summary, b: Summary) -> Option<Welch> {
    if a.n < 2 || b.n < 2 {
        return None;
    }
    let (na, nb) = (a.n as f64, b.n as f64);
    let (va, vb) = (a.sd.powi(2) / na, b.sd.powi(2) / nb);
    let diff_percent = (a.mean - b.mean) / b.mean * 100.;
    let pooled = (((na - 1.) * a.sd.powi(2) + (nb - 1.) * b.sd.powi(2)) / (na + nb - 2.)).sqrt();
    let correction = 1. - 3. / (4. * (na + nb) - 9.);
    let g = if pooled > 0. {
        (a.mean - b.mean) / pooled * correction
    } else {
        f64::INFINITY.copysign(a.mean - b.mean)
    };
    if va + vb == 0. {
        // no variance at all, any difference is exact
        let p = if a.mean == b.mean { 1. } else { 0. };
        return Some(Welch {
            diff_percent,
            t: f64::INFINITY.copysign(a.mean - b.mean),
            df: na + nb - 2.,
            p,
            g,
        });
    }
    let t = (a.mean - b.mean) / (va + vb).sqrt();
    let df = (va + vb).powi(2) / (va.powi(2) / (na - 1.) + vb.powi(2) / (nb - 1.));
    let dist = StudentsT::new(0., 1., df).ok()?;
    let p = 2. * (1. - dist.cdf(t.abs()));
    Some(Welch {
        diff_percent,
        t,
        df,
        p,
        g,
    })
}

/// Holm-Bonferroni adjusted p-values, in the order of `p`.
pub fn holm(p: &[f64]) -> Vec<f64> {
    let m = p.len();
    let mut order: Vec<usize> = (0..m).collect();
    order.sort_by(|&a, &b| p[a].total_cmp(&p[b]));
    let mut adjusted = vec![0.; m];
    let mut running_max: f64 = 0.;
    for (rank, &i) in order.iter().enumerate() {
        running_max = running_max.max(((m - rank) as f64 * p[i]).min(1.));
        adjusted[i] = running_max;
    }
    adjusted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn holm_matches_reference() {
        // statsmodels.stats.multitest.multipletests([0.01, 0.04, 0.03, 0.005], method="holm")
        let adjusted = holm(&[0.01, 0.04, 0.03, 0.005]);
        let expected = [0.03, 0.06, 0.06, 0.02];
        for (a, e) in adjusted.iter().zip(expected) {
            assert!((a - e).abs() < 1e-12, "{adjusted:?}");
        }
    }

    #[test]
    fn summary() {
        let s = Summary::of(&[2., 4., 4., 4., 5., 5., 7., 9.]).unwrap();
        assert_eq!(s.mean, 5.);
        assert!((s.sd - 2.138).abs() < 0.001);
    }

    #[test]
    fn welch_matches_reference() {
        // t = -1.9757, df = 7.711, p = 0.0850 (scipy.stats.ttest_ind(..., equal_var=False))
        let a = Summary::of(&[1., 2., 3., 4., 5.]).unwrap();
        let b = Summary::of(&[3., 4., 5., 6., 8.]).unwrap();
        let w = welch(a, b).unwrap();
        assert!((w.t + 1.9757).abs() < 0.001, "t = {}", w.t);
        assert!((w.df - 7.711).abs() < 0.001, "df = {}", w.df);
        assert!((w.p - 0.0850).abs() < 0.001, "p = {}", w.p);
    }
}
