use plotlib::repr::Plot;
use plotlib::view::ContinuousView;
use std::collections::HashMap;
use std::mem::replace;
use std::error::Error;
use yew::prelude::*;
use yahoo_finance::Bar;
use statrs::statistics::Statistics;
use plotlib::style::LineStyle;
use kelly::{ KellyAssumption, KellyFormulaBuilder };


struct Qtable {
    timestamp: Vec<i64>,
    reward: Vec<f64>
}

pub fn get_kelly(data: Vec<Bar>) -> (Result<Qtable, Qtable, Box<dyn Error>>) {
    let mut closedata: Qtable = Qtable { timestamp: Vec::new(), reward: Vec::new() };
    let init_vol: f64 = data[0].close.clone();
    for v in data.iter() {
        closedata.timestamp.push(v.timestamp);
        closedata.reward.push(v.close/init_vol)
    }
    let num_range: i64 = 20;
    let mmin: f64 = closedata.reward.min().clone();
    let mmax: f64 = closedata.reward.max().clone();
    let interval: f64 = (mmax - mmin)/(num_range as f64);
    let mut v0: Vec<f64> = closedata.reward.clone();
    v0.remove(v0.len()-1);
    let mut v1: Vec<f64> = closedata.reward.clone();
    v1.remove(0);
    let mut v0_v1: HashMap<f64, f64> = HashMap::new();
    for i in 0..v0.len() {
        v0_v1.insert(v0[i], (v1[i]-v0[i])/(v0[i]))
    }
    v0.clear();
    v1.clear();
    let assums: Vec<f64> = vec![0.; (num_range as usize)];
    for n in 0..num_range {
        let mut r_vec: Vec<f64> = Vec::new();
        for (k, v) in v0_v1.iter()
            .filter(|k | *k >= mmin + (n as f64)*interval && *k < mmin + ((n+1) as f64)*interval) {
                r_vec.push(*v)
            }
        if n == num_range-1 {
            r_vec.push(v0_v1.get(mmax))
        }
        replace(&mut assums[n as usize], r_vec.mean());
        r_vec.clear()
    }
    let mut kellydata: Qtable = Qtable {timestamp: closedata.timestamp.clone(), reward: vec![0; closedata.reward.len()]};
    replace(&mut kellydata.reward[0],init_vol);
    for k in 1..closedata.reward.len() {
        let rank: i64 = ((kellydata.reward[k-1] - mmin)/interval) as i64;
        let assumptions = vec![KellyAssumption(1., assums[rank as usize] as f32)];
        let kelly = KellyFormulaBuilder::new().set_assumptions(assumptions).calculate();
        replace(&mut kellydata.reward[k], ((kelly*((closedata.reward[k]/closedata.reward[k-1])*kellydata.reward[k-1]) as f32) + (1.-kelly)*kellydata.reward[k-1] as f32 ) as f64);
    }
    Ok((closedata, kellydata))
}

#[tokio::main]
#[function_component]
pub async fn Kplot() -> Html{
    let data = use_state::<Result>();
    let view = use_state::<ContinuousView>();

    let onclick = {
        let data = data.clone();
        Callback::from(move |_| {
            let (closedata, kellydata) = get_kelly(data);
            let line1: Plot = Plot::new(closedata).line_style(
                LineStyle::new()
                    .colour("#DD3355"),
            ).lengend("All-in Rewards");
            let line2: Plot = Plot::new(kellydata).line_style(
                LineStyle::new()
                    .colour("#35C788"),
            ).lengend("Kelly criterion Rewards");
            view.set(ContinuousView::new().add(line1).add(line2).unwrap());        
        })
    };

    html! {
        <>
            <canvas width={600} height={600} draw={view} />
            <div>
            <button {onclick}>{ "Plot chart" }</button>
            </div>
        </>
    }
}
