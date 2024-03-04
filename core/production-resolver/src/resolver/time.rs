pub struct FactoryTime {
    day: u32,
    slot: u32,
}

impl FactoryTime {
    pub const TIME_SLOTS_IN_DAY: u32 = 12;
    pub const DELIVERY_WINDOW: u32 = 9;

    pub fn new(day: u32, slot: u32) -> Self {
        Self { day, slot }
    }

    pub fn from_due_time(due_date: u32) -> Self {
        let day = due_date;
        let slot = Self::DELIVERY_WINDOW;
        Self { day, slot }
    }

    pub fn checked_sub(&self, rhs: FactoryTime) -> Option<FactoryTime> {
        let Self { day, slot } = self;

        let mut day = match day.checked_sub(rhs.day) {
            Some(d) => d,
            None => return None,
        };

        let slot = match slot.checked_sub(rhs.slot) {
            Some(s) => s,
            None => {
                if day <= 1 {
                    return None;
                }
                day -= 1;
                slot + Self::TIME_SLOTS_IN_DAY - rhs.slot
            }
        };

        Some(Self { day, slot })
    }
}

impl std::ops::Add for FactoryTime {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let Self { day, slot } = self;
        let mut day = day + rhs.day;
        let mut slot = slot + rhs.slot;

        if slot > Self::TIME_SLOTS_IN_DAY {
            day += 1;
            slot -= Self::TIME_SLOTS_IN_DAY;
        }

        Self { day, slot }
    }
}
