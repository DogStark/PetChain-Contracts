use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Timeframe {
    AllTime,
    Monthly,
    Weekly,
}

impl Timeframe {
    /// Returns the unix timestamp cutoff for the timeframe (seconds since epoch).
    /// `now` is the current unix timestamp in seconds.
    pub fn cutoff(&self, now: u64) -> Option<u64> {
        match self {
            Timeframe::AllTime => None,
            Timeframe::Monthly => Some(now.saturating_sub(30 * 24 * 3600)),
            Timeframe::Weekly => Some(now.saturating_sub(7 * 24 * 3600)),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct OrganizerScore {
    pub rank: usize,
    pub organizer: String,
    pub score: u64,
}

/// A single ticket-sale event attributed to an organizer at a point in time.
#[derive(Debug, Clone)]
pub struct TicketSale {
    pub organizer: String,
    pub tickets_sold: u64,
    pub timestamp: u64,
}

/// Rank organizers by total tickets sold within the given timeframe.
/// Returns a JSON-serialisable vec sorted by score descending.
pub fn rank_organizers(sales: &[TicketSale], timeframe: &Timeframe, now: u64) -> Vec<OrganizerScore> {
    let cutoff = timeframe.cutoff(now);

    let mut totals: HashMap<String, u64> = HashMap::new();
    for sale in sales {
        if let Some(c) = cutoff {
            if sale.timestamp < c {
                continue;
            }
        }
        *totals.entry(sale.organizer.clone()).or_insert(0) += sale.tickets_sold;
    }

    let mut ranked: Vec<(String, u64)> = totals.into_iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1));

    ranked
        .into_iter()
        .enumerate()
        .map(|(i, (organizer, score))| OrganizerScore {
            rank: i + 1,
            organizer,
            score,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now() -> u64 {
        1_000_000
    }

    fn sales() -> Vec<TicketSale> {
        vec![
            TicketSale { organizer: "Alice".into(), tickets_sold: 50, timestamp: now() - 1 },
            TicketSale { organizer: "Bob".into(),   tickets_sold: 30, timestamp: now() - 1 },
            TicketSale { organizer: "Alice".into(), tickets_sold: 20, timestamp: now() - 8 * 24 * 3600 }, // >7 days ago
            TicketSale { organizer: "Carol".into(), tickets_sold: 10, timestamp: now() - 31 * 24 * 3600 }, // >30 days ago
        ]
    }

    #[test]
    fn all_time_includes_all() {
        let result = rank_organizers(&sales(), &Timeframe::AllTime, now());
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].organizer, "Alice");
        assert_eq!(result[0].score, 70);
        assert_eq!(result[0].rank, 1);
    }

    #[test]
    fn monthly_excludes_old() {
        let result = rank_organizers(&sales(), &Timeframe::Monthly, now());
        // Carol's sale is >30 days old, excluded
        assert!(!result.iter().any(|r| r.organizer == "Carol"));
        assert_eq!(result[0].organizer, "Alice");
        assert_eq!(result[0].score, 50);
    }

    #[test]
    fn weekly_excludes_older_than_7_days() {
        let result = rank_organizers(&sales(), &Timeframe::Weekly, now());
        // Alice's old sale and Carol's sale excluded; Alice only has 50 recent
        let alice = result.iter().find(|r| r.organizer == "Alice").unwrap();
        assert_eq!(alice.score, 50);
        assert!(!result.iter().any(|r| r.organizer == "Carol"));
    }

    #[test]
    fn empty_sales_returns_empty() {
        let result = rank_organizers(&[], &Timeframe::AllTime, now());
        assert!(result.is_empty());
    }
}
