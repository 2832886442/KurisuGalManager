/// Bangumi API 集成测试脚本
/// 测试三个关键词的搜索、详情获取和关键词回退
/// 运行: cargo test -- --nocapture

#[cfg(test)]
mod tests {
    use crate::bangumi;

    #[tokio::test]
    async fn test_search_yuanzhikong() {
        println!("\n===== 测试搜索「缘之空」=====");
        match bangumi::search_games("缘之空").await {
            Ok(items) => {
                println!("找到 {} 条结果", items.len());
                for item in &items {
                    println!(
                        "  id={} name={} name_cn={} date={} score={} rank={} has_summary={}",
                        item.id,
                        item.name,
                        item.name_cn,
                        item.date,
                        item.score,
                        item.rank,
                        !item.summary.is_empty()
                    );
                }
                let found = items.iter().any(|i| i.id == 2388);
                assert!(found, "缘之空 (id=2388) 未出现在搜索结果中！");
                println!("✓ 缘之空 (id=2388) 在搜索结果中");
            }
            Err(e) => panic!("搜索失败: {}", e),
        }
    }

    #[tokio::test]
    async fn test_search_ever17() {
        println!("\n===== 测试搜索「ever17」=====");
        match bangumi::search_games("ever17").await {
            Ok(items) => {
                println!("找到 {} 条结果", items.len());
                for item in &items {
                    println!(
                        "  id={} name={} name_cn={} date={} score={} rank={}",
                        item.id, item.name, item.name_cn, item.date, item.score, item.rank
                    );
                }
                let found = items.iter().any(|i| i.id == 1126);
                assert!(found, "Ever17 原版 (id=1126) 未出现在搜索结果中！");
                println!("✓ Ever17 原版 (id=1126) 在搜索结果中");
            }
            Err(e) => panic!("搜索失败: {}", e),
        }
    }

    #[tokio::test]
    async fn test_search_youzhikong() {
        println!("\n===== 测试搜索「悠之空」=====");
        match bangumi::search_games("悠之空").await {
            Ok(items) => {
                println!("找到 {} 条结果", items.len());
                for item in &items {
                    println!(
                        "  id={} name={} name_cn={} date={} score={} rank={}",
                        item.id, item.name, item.name_cn, item.date, item.score, item.rank
                    );
                }
                let found = items.iter().any(|i| i.id == 6967);
                assert!(found, "悠之空 (id=6967) 未出现在搜索结果中！");
                println!("✓ 悠之空 (id=6967) 在搜索结果中");
            }
            Err(e) => panic!("搜索失败: {}", e),
        }
    }

    #[tokio::test]
    async fn test_detail_ever17() {
        println!("\n===== 测试获取 Ever17 (id=1126) 详情 =====");
        match bangumi::get_game_detail(1126).await {
            Ok(detail) => {
                println!("  name: {}", detail.name);
                println!("  name_cn: {}", detail.name_cn);
                println!("  summary_len: {}", detail.summary.len());
                println!("  date: {}", detail.date);
                println!("  score: {}", detail.score);
                println!("  image: {}", detail.image_large);
                println!("  tags count: {}", detail.tags.len());
                println!("  platform: {}", detail.platform);
                assert!(!detail.name.is_empty(), "name 不应为空");
                assert!(!detail.summary.is_empty(), "summary 不应为空");
                assert!(!detail.image_large.is_empty(), "image 不应为空");
                println!("✓ Ever17 详情获取成功");
            }
            Err(e) => panic!("获取详情失败: {}", e),
        }
    }

    /// 测试 NSFW 游戏的详情获取：v0 和旧 API 均返回 404，应通过搜索回退
    #[tokio::test]
    async fn test_detail_nsfw_with_search_fallback() {
        println!("\n===== NSFW游戏详情获取（关键词搜索回退）=====");

        // 缘之空 (id=2388)：NSFW，详情 API 封禁，需要通过搜索回退
        println!("--- 测试 缘之空 (id=2388) 搜索回退 ---");
        match bangumi::search_games("缘之空").await {
            Ok(results) => {
                let item = results.iter().find(|i| i.id == 2388);
                assert!(item.is_some(), "缘之空(id=2388)应在搜索结果中");
                let item = item.unwrap();
                println!(
                    "  搜索找到: name={} name_cn={} summary_len={} image={}",
                    item.name,
                    item.name_cn,
                    item.summary.len(),
                    item.image_large
                );
                assert!(!item.name_cn.is_empty(), "name_cn 不应为空");
                assert!(!item.summary.is_empty(), "summary 不应为空");
                assert!(!item.image_large.is_empty(), "image_large 不应为空");
                println!("  ✓ 搜索回退数据完整可用");
            }
            Err(e) => panic!("搜索回退失败: {}", e),
        }

        // 悠之空 (id=6967)：NSFW，同上
        println!("--- 测试 悠之空 (id=6967) 搜索回退 ---");
        match bangumi::search_games("悠之空").await {
            Ok(results) => {
                let item = results.iter().find(|i| i.id == 6967);
                assert!(item.is_some(), "悠之空(id=6967)应在搜索结果中");
                let item = item.unwrap();
                println!(
                    "  搜索找到: name={} name_cn={} summary_len={} image={}",
                    item.name,
                    item.name_cn,
                    item.summary.len(),
                    item.image_large
                );
                assert!(!item.name_cn.is_empty(), "name_cn 不应为空");
                assert!(!item.summary.is_empty(), "summary 不应为空");
                println!("  ✓ 搜索回退数据完整可用");
            }
            Err(e) => panic!("搜索回退失败: {}", e),
        }
    }

    /// 完整流程测试：搜索 → 获取详情 → 下载封面
    /// 模拟 commands::fetch_bangumi_game 的完整逻辑
    #[tokio::test]
    async fn test_full_flow_all_three() {
        println!("\n===== 完整流程测试：三个关键词 =====");

        // === Ever17 (正常路径：v0 API) ===
        println!("\n--- Ever17 (id=1126)：正常 v0 API 路径 ---");
        let detail_1126 = bangumi::get_game_detail(1126)
            .await
            .expect("Ever17 详情获取失败");
        println!("  名称: {} / {}", detail_1126.name, detail_1126.name_cn);
        println!("  简介: {} 字", detail_1126.summary.len());
        println!("  标签: {} 个", detail_1126.tags.len());
        println!("  封面: {}", detail_1126.image_large);
        assert!(!detail_1126.name.is_empty());
        assert!(!detail_1126.summary.is_empty());
        assert!(!detail_1126.image_large.is_empty());
        println!("  ✓ Ever17 完整数据获取成功");

        // 下载封面
        match bangumi::download_cover_as_base64(&detail_1126.image_large).await {
            Ok(data_uri) => println!("  ✓ 封面下载成功: {} chars", data_uri.len()),
            Err(e) => println!("  ⚠ 封面下载失败（非致命）: {}", e),
        }

        // === 缘之空 (NSFW路径：搜索回退) ===
        println!("\n--- 缘之空 (id=2388)：NSFW 搜索回退路径 ---");
        let search_results = bangumi::search_games("缘之空")
            .await
            .expect("缘之空搜索失败");
        let item_2388 = search_results
            .iter()
            .find(|i| i.id == 2388)
            .expect("缘之空(id=2388)不在搜索结果中");
        println!(
            "  搜索数据: name={} name_cn={}",
            item_2388.name, item_2388.name_cn
        );
        println!("  简介: {} 字", item_2388.summary.len());
        println!("  封面URL: {}", item_2388.image_large);
        assert!(!item_2388.name_cn.is_empty());
        assert!(!item_2388.summary.is_empty());
        println!("  ✓ 缘之空搜索回退数据完整");

        // 下载封面
        match bangumi::download_cover_as_base64(&item_2388.image_large).await {
            Ok(data_uri) => println!("  ✓ 封面下载成功: {} chars", data_uri.len()),
            Err(e) => println!("  ⚠ 封面下载失败（非致命）: {}", e),
        }

        // === 悠之空 (NSFW路径：搜索回退) ===
        println!("\n--- 悠之空 (id=6967)：NSFW 搜索回退路径 ---");
        let search_results = bangumi::search_games("悠之空")
            .await
            .expect("悠之空搜索失败");
        let item_6967 = search_results
            .iter()
            .find(|i| i.id == 6967)
            .expect("悠之空(id=6967)不在搜索结果中");
        println!(
            "  搜索数据: name={} name_cn={}",
            item_6967.name, item_6967.name_cn
        );
        println!("  简介: {} 字", item_6967.summary.len());
        println!("  封面URL: {}", item_6967.image_large);
        assert!(!item_6967.name_cn.is_empty());
        assert!(!item_6967.summary.is_empty());
        println!("  ✓ 悠之空搜索回退数据完整");

        // 下载封面
        match bangumi::download_cover_as_base64(&item_6967.image_large).await {
            Ok(data_uri) => println!("  ✓ 封面下载成功: {} chars", data_uri.len()),
            Err(e) => println!("  ⚠ 封面下载失败（非致命）: {}", e),
        }

        println!("\n===== 全部流程测试通过 =====");
    }
}
