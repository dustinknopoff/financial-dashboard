#![allow(dead_code)]
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum Balance {
    Account(BalanceListing),
    Total(Commodity),
}

#[derive(Debug, Deserialize, Clone)]
pub struct BalanceListing((String, String, usize, Vec<Commodity>));

#[derive(Debug, Deserialize, Clone)]
pub struct Commodity {
    acommodity: String,
    aprice: Option<PriceListing>,
    aquantity: Quantity,
    astyle: Style,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]

pub struct Quantity {
    decimal_mantissa: isize,
    decimal_places: isize,
    floating_point: f32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Style {
    #[serde(rename = "ascommodityside")]
    as_commidity_side: Side,
    #[serde(rename = "ascommodityspaced")]
    as_commodity_spaced: bool,
    #[serde(rename = "asdecimalpoint")]
    as_decimal_point: String,
    #[serde(rename = "asdigitgroups")]
    as_digits_group: Option<(String, Vec<usize>)>,
    #[serde(rename = "asprecision")]
    as_precision: usize,
}

#[derive(Debug, Clone)]
pub enum Side {
    Left,
    Right,
}

impl ::serde::Serialize for Side {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        serializer.serialize_str(match *self {
            Side::Left => "L",
            Side::Right => "R",
        })
    }
}

impl<'d> ::serde::Deserialize<'d> for Side {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'d>,
    {
        struct Visitor;

        impl ::serde::de::Visitor<'_> for Visitor {
            type Value = Side;

            fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                write!(formatter, "a string for {}", stringify!($name))
            }

            fn visit_str<E>(self, value: &str) -> Result<Side, E>
            where
                E: ::serde::de::Error,
            {
                match value {
                    "L" => Ok(Side::Left),
                    "R" => Ok(Side::Right),
                    _ => Err(E::invalid_value(
                        ::serde::de::Unexpected::Other(&format!(
                            "unknown {} variant: {}",
                            stringify!($name),
                            value
                        )),
                        &self,
                    )),
                }
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct PriceListing {
    contents: Box<Commodity>,
    tag: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PeriodReportSpan((String, String));

#[derive(Debug, Deserialize, Clone)]
pub struct PeriodReportRow {
    #[serde(rename = "prrAmounts")]
    amounts: Vec<Vec<Commodity>>,
    #[serde(rename = "prrAverage")]
    averages: Vec<Commodity>,
    #[serde(rename = "prrName")]
    name: String,
    #[serde(rename = "prrTotal")]
    totals: Vec<Commodity>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PeriodReportTotal {
    #[serde(rename = "prrAmounts")]
    amounts: Vec<Vec<Commodity>>,
    #[serde(rename = "prrAverage")]
    averages: Vec<Commodity>,
    #[serde(rename = "prrName")]
    name: Vec<String>,
    #[serde(rename = "prrTotal")]
    totals: Vec<Commodity>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PeriodReport {
    #[serde(rename = "prDates")]
    dates: Vec<PeriodReportSpan>,
    #[serde(rename = "prRows")]
    rows: Vec<PeriodReportRow>,
    #[serde(rename = "prTotals")]
    totals: PeriodReportTotal,
}

impl PeriodReport {
    pub fn num_dates(&self) -> usize {
        self.dates.len()
    }

    pub fn total_at_index(&self, index: usize) -> Vec<AccountValue> {
        if self.totals.amounts[index].is_empty() {
            vec![AccountValue {
                amount: 0.0,
                commodity: String::from("USD"),
                date_range: self.dates[index].clone(),
            }]
        } else {
            self.totals.amounts[index]
                .clone()
                .into_iter()
                .enumerate()
                .map(|(index, value)| AccountValue {
                    amount: value.aquantity.floating_point,
                    commodity: value.acommodity,
                    date_range: self.dates[index].clone(),
                })
                .collect()
        }
    }
}

#[derive(Debug, Clone)]
pub struct AccountValue {
    pub amount: f32,
    pub commodity: String,
    pub date_range: PeriodReportSpan,
}

#[derive(Debug, Clone)]
pub struct ExpenseReport {
    pub amount: f32,
    pub label: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BalanceAccount(Vec<Vec<Balance>>);

impl BalanceAccount {
    pub fn new(balance: Vec<Vec<Balance>>) -> BalanceAccount {
        BalanceAccount(balance)
    }

    pub fn get_totals(&self) -> Vec<AccountValue> {
        self.0
            .last()
            .unwrap()
            .clone()
            .into_iter()
            .map(|total| match total {
                Balance::Account(_) => anyhow::bail!("Last row must be a Total!"),
                Balance::Total(total) => Ok(AccountValue {
                    amount: total.aquantity.floating_point,
                    commodity: total.acommodity,
                    date_range: PeriodReportSpan((String::new(), String::new())),
                }),
            })
            .filter_map(|total| total.ok())
            .collect()
    }

    pub fn get_expense_report(&self) -> Vec<ExpenseReport> {
        let total = if self.get_totals().is_empty() {
            panic!("No expenses this month to calculate a total from!");
        } else {
            let tmp = &self.get_totals()[0];
            tmp.clone()
        };
        self.0[0]
            .clone()
            .into_iter()
            .map(|account| match account {
                Balance::Account(account) => Ok(ExpenseReport {
                    label: account.0 .1.clone(),
                    amount: account.0 .3[0].aquantity.floating_point / total.amount,
                }),
                Balance::Total(_) => anyhow::bail!("First list must be accounts!"),
            })
            .filter_map(|acc| acc.ok())
            .collect()
    }
}
