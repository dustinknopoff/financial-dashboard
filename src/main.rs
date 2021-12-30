use ansi_term::Colour;
use csv::{Reader, StringRecord};
use piechart::Color;
use textplots::{Chart, Plot, Shape};
use xshell::cmd;

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
    // Fetch only total;
    let all_expenses = match all_expenses
        .iter()
        .last() {
            Some(val) => val,
            None => anyhow::bail!("No expenses recorded. Calculating savings rate relies on comparisons between income and expenses.")
        };
    let all_income = fetch_balance("Income")?;
    // Fetch only total
    let all_income = match all_income.iter().last() {
        Some(val) => val,
        None => anyhow::bail!("No income recorded. Calculating savings rate relies on comparisons between income and expenses.")
    };

    // Calculate
    let mut cummulative_rate = 0_f32;
    let max_capacity = all_expenses.len();
    let mut rates: Vec<(f32, f32)> = Vec::with_capacity(max_capacity - 1);
    let mut avg_daily_expense = 0_f32;
    let mut avg_daily_income = 0_f32;
    let num_months = all_expenses.len() - 2;
    for (index, value) in all_expenses.iter().enumerate() {
        // Columns for each month, skipping first column (account name) and last (total)
        // Get as numbers. Equation is (income - expenses) / income
        if index != 0 && index <= num_months {
            let expenses: f32 = string_to_f32(value)?;
            let income: f32 = string_to_f32(all_income.iter().nth(index).unwrap())?;
            let rate = (income - expenses) / income;
            let rate = if rate.is_nan() { 0_f32 } else { rate };
            rates.push(((index - 1) as f32, rate * 100_f32));
            avg_daily_expense += expenses / 30_f32;
            avg_daily_income += income / 30_f32;
            cummulative_rate += rate;
        }
    }
    avg_daily_income /= (num_months) as f32;
    avg_daily_expense /= (num_months) as f32;
    cummulative_rate /= (num_months) as f32;
    let fire = avg_daily_expense * 365_f32 * 25_f32;
    let aaw = ((avg_daily_income * 365_f32 * 23_f32) / 10_f32 / 2_f32 - liabilities).abs();
    let paw = (((avg_daily_income * 365_f32 * 23_f32) / 10_f32) * 2_f32 - liabilities).abs();

    let expenses_breakdown = fetch_expenses_this_month()?;
    // Get final row/column as number
    let total: f32 = string_to_f32(expenses_breakdown.last().unwrap().iter().last().unwrap())?;
    let data: Vec<piechart::Data> = expenses_breakdown[..expenses_breakdown.len() - 1]
        .iter()
        .enumerate()
        .map(|(index, expense)| piechart::Data {
            label: expense.get(0).unwrap().into(),
            // Value / total is percentage of total
            value: string_to_f32(expense.get(1).unwrap()).unwrap() / total,
            fill: fills[index % 5],
            color: Some(colors[index % 5].into()),
        })
        .collect();

    // Present
    println!("savings rate: last {} months", num_months);
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

fn fetch_balance(account_prefix: &str) -> anyhow::Result<Vec<StringRecord>> {
    let output =
        cmd!("hledger bal ^{account_prefix} -O csv -M -b lastquarter -C -U -T -X USD").read()?;
    let mut rdr = Reader::from_reader(output.as_bytes());
    Ok(rdr.records().filter_map(|record| record.ok()).collect())
}

fn fetch_expenses_this_month() -> anyhow::Result<Vec<StringRecord>> {
    let output = cmd!("hledger bal ^Expenses  --begin thismonth -O csv -X USD").read()?;
    let mut rdr = Reader::from_reader(output.as_bytes());
    Ok(rdr.records().filter_map(|record| record.ok()).collect())
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
