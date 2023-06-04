pub async fn scan(&mut self) -> Result<impl Stream<Item = DiscoveredHub> + '_> {
    let events = self.adapter.events().await?;
    self.adapter.start_scan(ScanFilter::default()).await?;
    Ok(events.filter_map(|event| async {
        let CentralEvent::DeviceDiscovered(id) = event else { None? };
        // get peripheral info
        let peripheral = self.adapter.peripheral(&id).await.ok()?;
        println!("{:?}", peripheral.properties().await.unwrap());
        let Some(props) = peripheral.properties().await.ok()? else { None? };
        if let Some(hub_type) = identify_hub(&props).await.ok()? {
            let hub = DiscoveredHub {
                hub_type,
                addr: id,
                name: props
                    .local_name
                    .unwrap_or_else(|| "unknown".to_string()),
            };
            Some(hub)
        } else { None }
    }))
}


btleplug::winrtble::peripheral::Peripheral
fn notifications<'life0, 'async_trait>(&'life0 self) 
-> ::core::pin::Pin<Box<dyn ::core::future::Future
<Output = Result<Pin<Box<dyn Stream<Item = ValueNotification> + Send>>>> 
+ ::core::marker::Send + 'async_trait>>
where
    'life0: 'async_trait,
    Self: 'async_trait,
    