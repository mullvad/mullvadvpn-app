use super::interface::Family;

#[derive(Clone, Debug)]
pub struct IpMap<T> {
    v4: Option<T>,
    v6: Option<T>,
}

impl<T> IpMap<T> {
    pub const fn new() -> Self {
        Self { v4: None, v6: None }
    }

    pub fn get(&self, family: Family) -> Option<&T> {
        match family {
            Family::V4 => self.v4.as_ref(),
            Family::V6 => self.v6.as_ref(),
        }
    }

    /// Insert an option value and return the old value.
    pub fn set(&mut self, family: Family, value: Option<T>) -> Option<T> {
        let old_value = self.remove(family);
        match family {
            Family::V4 => self.v4 = value,
            Family::V6 => self.v6 = value,
        };
        old_value
    }

    /// Insert a value and return the old value.
    pub fn insert(&mut self, family: Family, value: T) -> Option<T> {
        match family {
            Family::V4 => self.v4.replace(value),
            Family::V6 => self.v6.replace(value),
        }
    }

    /// Remove a value and return it.
    pub fn remove(&mut self, family: Family) -> Option<T> {
        match family {
            Family::V4 => self.v4.take(),
            Family::V6 => self.v6.take(),
        }
    }

    pub fn len(&self) -> usize {
        self.iter().count()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> impl Iterator<Item = (Family, &T)> {
        [(Family::V4, &self.v4), (Family::V6, &self.v6)]
            .into_iter()
            .flat_map(|(family, elem)| Some((family, elem.as_ref()?)))
    }
}
