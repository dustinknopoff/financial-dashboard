use ansi_term::Colour;
use piechart::Color;
use textplots::{Chart, Plot, Shape};
use types::{Balance, PeriodReport};
use xshell::cmd;

use crate::types::BalanceAccount;

mod types;

fn main() -> anyhow::Result<()> {
    // Constants
    let fills = ['•', '▪', '▴', '░', '▀'];
    let colors = [
        Color::Blue,
        Color::Red,
        Color::Green,
        Color::Yellow,
        Color::Purple,
    ];

    // Fetch via hledger
    let liabilities = fetch_liabilities()?;
    let all_expenses = fetch_balance("Expenses")?;
    let all_income = fetch_balance("Income")?;

    let mut cummulative_rate = 0_f32;
    let mut rates: Vec<(f32, f32)> = Vec::with_capacity(all_expenses.num_dates());
    let mut avg_daily_expense = 0_f32;
    let mut avg_daily_income = 0_f32;
    for row in 0..all_expenses.num_dates() {
        let expenses = &all_expenses.total_at_index(row)[0];
        let income = &all_income.total_at_index(row)[0];
        let rate = (income.amount - expenses.amount) / income.amount;
        // If it's NaN, return 0
        let rate = if rate.is_nan() { 0_f32 } else { rate };
        // Push to our List for displaying as a step graph
        rates.push((row as f32, rate * 100_f32));
        // Sum average daily expenses and income hard coded to a month as 30 days
        avg_daily_expense += expenses.amount / 30_f32;
        avg_daily_income += income.amount / 30_f32;
        // Add to cummulative_rate
        cummulative_rate += rate;
    }
    // Calculate average daily income over months in this + last quarter
    avg_daily_income /= (all_expenses.num_dates()) as f32;
    avg_daily_expense /= (all_expenses.num_dates()) as f32;
    // calculate average savgins rate over months in this + last quarter
    cummulative_rate /= (all_expenses.num_dates()) as f32;
    let fire = avg_daily_expense * 365_f32 * 25_f32;
    let aaw = ((avg_daily_income * 365_f32 * 23_f32) / 10_f32 / 2_f32 - liabilities).abs();
    let paw = (((avg_daily_income * 365_f32 * 23_f32) / 10_f32) * 2_f32 - liabilities).abs();

    let expenses_breakdown = BalanceAccount::new(fetch_expenses_this_month()?);
    let data: Vec<piechart::Data> = expenses_breakdown
        .get_expense_report()
        .into_iter()
        .enumerate()
        .map(|(index, expense)| piechart::Data {
            label: expense.label,
            // Value / total is percentage of total
            value: expense.amount,
            fill: fills[index % 5],
            color: Some(colors[index % 5].into()),
        })
        .collect();

    // // Present
    println!("savings rate: last {} months", all_expenses.num_dates());
    Chart::new(120, 60, 0.0, 8.0)
        .lineplot(&Shape::Steps(&rates))
        .nice();
    println!(
        "Average Savings Rate: {}",
        current_rate_colored(cummulative_rate * 100_f32)
    );
    piechart::Chart::new()
        .radius(9)
        .aspect_ratio(3)
        .legend(true)
        .draw(&data);
    println!("\nFinancial Independence, Retire Early: ${}", fire);
    println!("Average-Accumulator of Wealth: ${}", aaw);
    println!("Prodigious-Accumulator of Wealth: ${}", paw);
    Ok(())
}

fn fetch_balance(account_prefix: &str) -> anyhow::Result<PeriodReport> {
    let output =
        cmd!("hledger bal ^{account_prefix} -O json -M -b lastquarter -C -U -X USD").read()?;
    Ok(serde_json::from_str(&output).expect("Failed to parse the balance"))
}

fn fetch_expenses_this_month() -> anyhow::Result<Vec<Vec<Balance>>> {
    let output = cmd!("hledger bal ^Expenses  --begin thismonth -O json -X USD").read()?;
    Ok(serde_json::from_str(&output).expect("Failed to retrieve expenses this month"))
}

fn fetch_liabilities() -> anyhow::Result<f32> {
    let output = cmd!("hledger bal Liabilities --format '%(total)' -X USD").read()?;
    let output = output.split('\n').last().unwrap();
    string_to_f32(output)
}

fn current_rate_colored(value: f32) -> String {
    let as_percent = &format!("{}%", value);
    if value <= 0_f32 {
        return Colour::Red.paint(as_percent).to_string();
    } else if value > 0_f32 && value <= 50.0 {
        return Colour::Yellow.paint(as_percent).to_string();
    } else {
        return Colour::Green.paint(as_percent).to_string();
    }
}

fn string_to_f32(src: &str) -> anyhow::Result<f32> {
    Ok(src.replace(" USD", "").replace(",", "").parse::<f32>()?)
}
