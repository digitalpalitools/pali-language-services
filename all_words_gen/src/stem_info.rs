use pls_core_extras::sql_access::SqlAccess;
use std::fmt;

#[derive(Debug)]
pub struct StemInfo {
    pub row_id: i64,
    pub pali1: String,
}

pub struct StemInfoIterator<'a> {
    sql_access: &'a SqlAccess,
    max_stems_to_fetch: i64,
    max_batch_size: i64,
    pub stems_fetched: i64,
    pub batches_fetched: i64,
    pub error: Option<String>,
}

impl fmt::Display for StemInfoIterator<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Iterator state:")?;
        writeln!(f, "... max_stems_to_fetch: {}", self.max_stems_to_fetch)?;
        writeln!(f, "... max_batch_size: {}", self.max_batch_size)?;
        writeln!(f, "... stems_fetched: {}", self.stems_fetched)?;
        writeln!(f, "... batches_fetched: {}", self.batches_fetched)?;
        writeln!(f, "... error: {:?}", self.error)?;
        Ok(())
    }
}

impl<'a> StemInfoIterator<'a> {
    pub fn new(sql_access: &'a SqlAccess, max_stems_to_fetch: i64, max_batch_size: i64) -> Self {
        StemInfoIterator {
            sql_access,
            max_stems_to_fetch,
            max_batch_size,
            stems_fetched: 0,
            batches_fetched: 0,
            error: None,
        }
    }
}

impl Iterator for StemInfoIterator<'_> {
    type Item = Vec<StemInfo>;

    fn next(&mut self) -> Option<Self::Item> {
        let stems_remaining = self.max_stems_to_fetch - self.stems_fetched;
        let batch_size = if stems_remaining < self.max_batch_size {
            stems_remaining
        } else {
            self.max_batch_size
        };

        let curr_batch = self
            .sql_access
            .exec(&format!(
                "SELECT CAST(rowid AS TEXT), pāli1 FROM '_stems' order by rowid LIMIT {},{}",
                self.stems_fetched, batch_size
            ))
            .unwrap_or_else(|x| {
                self.error = Some(x);
                vec![]
            });

        if curr_batch.len() == 1
            && !curr_batch[0].is_empty()
            && (self.stems_fetched < self.max_stems_to_fetch)
        {
            self.batches_fetched += 1;
            let stem_infos: Vec<StemInfo> = curr_batch[0]
                .to_owned()
                .into_iter()
                .map(|x| StemInfo {
                    row_id: x[0]
                        .parse::<i64>()
                        .expect("rowid must be i64. check for db corruption."),
                    pali1: x[1].to_owned(),
                })
                .collect();
            self.stems_fetched += stem_infos.len() as i64;

            Some(stem_infos)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pls_core_extras::sql_access::create_sql_access;

    #[test]
    fn test_iterator_uneven_batches() {
        let sa = create_sql_access();
        let mut sii = StemInfoIterator::new(&sa, 1000, 11);
        for _si in &mut sii {}

        assert_eq!(sii.max_stems_to_fetch, 1000);
        assert_eq!(sii.max_batch_size, 11);
        assert_eq!(sii.stems_fetched, 1000);
        assert_eq!(sii.batches_fetched, 91);
        assert_eq!(sii.error, None);
    }

    #[test]
    fn test_iterator_even_batches() {
        let sa = create_sql_access();
        let mut sii = StemInfoIterator::new(&sa, 1000, 10);
        for _si in &mut sii {}

        assert_eq!(sii.max_stems_to_fetch, 1000);
        assert_eq!(sii.max_batch_size, 10);
        assert_eq!(sii.stems_fetched, 1000);
        assert_eq!(sii.batches_fetched, 100);
        assert_eq!(sii.error, None);
    }

    #[test]
    fn test_iterator_trailing_batches() {
        let sa = create_sql_access();
        let mut sii = StemInfoIterator::new(&sa, 25, 10);
        for _si in &mut sii {}

        assert_eq!(sii.max_stems_to_fetch, 25);
        assert_eq!(sii.max_batch_size, 10);
        assert_eq!(sii.stems_fetched, 25);
        assert_eq!(sii.batches_fetched, 3);
        assert_eq!(sii.error, None);
    }
}
