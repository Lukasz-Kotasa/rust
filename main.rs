pub fn max_profit(prices: Vec<i32>) -> i32 {
    let mut j: i32 = prices.len() as i32 - 1;
    let mut i: i32 = j - 1;
    let mut res = 0;
    
    while i >= 0 
    {
        if prices[j as usize]-prices[i as usize] > 0
        {
            res += prices[j as usize]-prices[i as usize];
            j-=1;
        } else
        {
            j=i;
        }
        i-=1;
    }
    res
}

fn main() {
    let prices1 = vec![7,1,5,3,6,4];
    let prices2 = vec![1,2,3,4,5];
    let prices3 = vec![7,6,4,3,1];
    let prices4 = vec![1,1,2,1,1];
    let prices5 = vec![1,3,2,1,3];
    let prices6 = vec![8,1,4,4,6];
    
    
    assert_eq!(max_profit(prices1), 7);
    assert_eq!(max_profit(prices2), 4);
    assert_eq!(max_profit(prices3), 0);
    assert_eq!(max_profit(prices4), 1);
    assert_eq!(max_profit(prices5), 4);
    assert_eq!(max_profit(prices6), 5);

    println!("Max profit tests passed");
}
