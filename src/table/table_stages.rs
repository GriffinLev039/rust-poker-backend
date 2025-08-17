#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TableStage {
    PreFlop,
    Flop,
    Turn,
    River,
    Showdown,
}

impl TableStage {
    pub fn get_next(self) -> TableStage {
        match self {
            TableStage::PreFlop => TableStage::Flop,
            TableStage::Flop => TableStage::Turn,
            TableStage::Turn => TableStage::River,
            TableStage::River => TableStage::Showdown,
            TableStage::Showdown => TableStage::Showdown,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn get_next_test() {
        let mut table_stage = TableStage::PreFlop;
        assert_eq!(table_stage.get_next(), TableStage::Flop);
        table_stage = table_stage.get_next();
        assert_eq!(table_stage.get_next(), TableStage::Turn);
        table_stage = table_stage.get_next();
        assert_eq!(table_stage.get_next(), TableStage::River);
        table_stage = table_stage.get_next();
        assert_eq!(table_stage.get_next(), TableStage::Showdown);
        table_stage = table_stage.get_next();
        assert_eq!(table_stage.get_next(), TableStage::Showdown);
    }
}
