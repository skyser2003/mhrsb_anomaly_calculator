use super::types::PointsVec;

pub struct CalcPoint {}

impl CalcPoint {
    pub fn is_possible_static(avail_points: &PointsVec, req_points: &PointsVec) -> bool {
        for (avail_point, req_point) in avail_points.iter().zip(req_points) {
            if avail_point < req_point {
                return false;
            }
        }

        true
    }
}
