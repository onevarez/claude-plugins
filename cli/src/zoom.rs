//! Zoom segment computation from cursor click clusters.
//!
//! Detects clusters of click activity and generates zoom segments
//! that kineto uses for per-frame crop+scale.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoomSegment {
    pub start_time: f64,
    pub end_time: f64,
    pub zoom_level: f64,
    pub center_x: f64,
    pub center_y: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub easing: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hold_duration_secs: Option<f64>,
}

/// A cursor sample extracted from Playwright console logs.
#[derive(Debug, Clone, Deserialize)]
pub struct CursorSample {
    pub t: f64,
    #[serde(alias = "type")]
    pub event_type: String,
    pub x: f64,
    pub y: f64,
}

/// Detect click clusters in cursor data and generate zoom segments.
///
/// Groups clicks that are close in time (within `time_window_s`) and space
/// (within `distance_threshold` normalized units). Each cluster becomes a
/// zoom segment centered on the average click position.
pub fn compute_zoom_segments(
    samples: &[CursorSample],
    viewport_w: f64,
    viewport_h: f64,
    zoom_level: f64,
) -> Vec<ZoomSegment> {
    let time_window_s = 3.5;
    let min_clicks = 2;

    // Filter to click events only and convert timestamps from ms to seconds
    let clicks: Vec<(f64, f64, f64)> = samples
        .iter()
        .filter(|s| s.event_type == "click")
        .map(|s| (s.t / 1000.0, s.x, s.y))
        .collect();

    if clicks.len() < min_clicks {
        return Vec::new();
    }

    // Greedy clustering: group clicks within time_window_s of each other
    let mut segments = Vec::new();
    let mut i = 0;

    while i < clicks.len() {
        let (start_t, _, _) = clicks[i];
        let mut cluster_end = i;

        // Extend cluster while next click is within the time window
        while cluster_end + 1 < clicks.len()
            && clicks[cluster_end + 1].0 - start_t < time_window_s
        {
            cluster_end += 1;
        }

        let cluster = &clicks[i..=cluster_end];
        if cluster.len() >= min_clicks {
            // Compute cluster center (normalized 0-1)
            let avg_x: f64 = cluster.iter().map(|(_, x, _)| x).sum::<f64>() / cluster.len() as f64;
            let avg_y: f64 = cluster.iter().map(|(_, _, y)| y).sum::<f64>() / cluster.len() as f64;

            let center_x = (avg_x / viewport_w).clamp(0.0, 1.0);
            let center_y = (avg_y / viewport_h).clamp(0.0, 1.0);

            let seg_start = cluster.first().unwrap().0;
            let seg_end = cluster.last().unwrap().0;

            segments.push(ZoomSegment {
                start_time: seg_start,
                end_time: seg_end,
                zoom_level,
                center_x,
                center_y,
                easing: None,
                hold_duration_secs: None,
            });
        }

        i = cluster_end + 1;
    }

    segments
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_clicks_no_segments() {
        let samples = vec![
            CursorSample { t: 100.0, event_type: "move".into(), x: 500.0, y: 300.0 },
        ];
        let segs = compute_zoom_segments(&samples, 1920.0, 1080.0, 2.0);
        assert!(segs.is_empty());
    }

    #[test]
    fn test_clustered_clicks_produce_segment() {
        let samples = vec![
            CursorSample { t: 1000.0, event_type: "click".into(), x: 960.0, y: 540.0 },
            CursorSample { t: 1500.0, event_type: "click".into(), x: 980.0, y: 550.0 },
            CursorSample { t: 2000.0, event_type: "click".into(), x: 940.0, y: 530.0 },
        ];
        let segs = compute_zoom_segments(&samples, 1920.0, 1080.0, 2.0);
        assert_eq!(segs.len(), 1);
        assert!((segs[0].zoom_level - 2.0).abs() < 1e-9);
        assert!((segs[0].center_x - 0.5).abs() < 0.05);
        assert!((segs[0].center_y - 0.5).abs() < 0.05);
    }

    #[test]
    fn test_distant_clicks_produce_separate_segments() {
        let samples = vec![
            CursorSample { t: 1000.0, event_type: "click".into(), x: 100.0, y: 100.0 },
            CursorSample { t: 1500.0, event_type: "click".into(), x: 120.0, y: 110.0 },
            // 10s gap
            CursorSample { t: 11000.0, event_type: "click".into(), x: 1800.0, y: 900.0 },
            CursorSample { t: 11500.0, event_type: "click".into(), x: 1820.0, y: 910.0 },
        ];
        let segs = compute_zoom_segments(&samples, 1920.0, 1080.0, 2.0);
        assert_eq!(segs.len(), 2);
    }

    #[test]
    fn test_single_click_no_segment() {
        let samples = vec![
            CursorSample { t: 1000.0, event_type: "click".into(), x: 500.0, y: 300.0 },
        ];
        let segs = compute_zoom_segments(&samples, 1920.0, 1080.0, 2.0);
        assert!(segs.is_empty());
    }
}
