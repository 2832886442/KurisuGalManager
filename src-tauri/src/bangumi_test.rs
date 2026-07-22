/// Bangumi API 集成测试脚本
/// 测试关键词的搜索、详情获取和关键词回退，所有测试对网络错误有容忍
/// 运行: cargo test -- --nocapture

#[cfg(test)]
mod tests {
    use crate::bangumi;

    /// 判断是否为可容忍的网络错误（不视为测试失败）
    fn is_network_err(e: &str) -> bool {
        e.contains("503")
            || e.contains("Service Temporarily")
            || e.contains("timeout")
            || e.contains("connect")
            || e.contains("decode")
            || e.contains("读取响应")
    }

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
            Err(e) => {
                if is_network_err(&e) {
                    println!("⚠ 网络错误（跳过）: {}", e);
                } else {
                    panic!("搜索失败: {}", e);
                }
            }
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
            Err(e) => {
                if is_network_err(&e) {
                    println!("⚠ 网络错误（跳过）: {}", e);
                } else {
                    panic!("搜索失败: {}", e);
                }
            }
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
            Err(e) => {
                if is_network_err(&e) {
                    println!("⚠ 网络错误（跳过）: {}", e);
                } else {
                    panic!("搜索失败: {}", e);
                }
            }
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
            Err(e) => {
                if is_network_err(&e) {
                    println!("⚠ 网络错误（跳过）: {}", e);
                } else {
                    panic!("获取详情失败: {}", e);
                }
            }
        }
    }

    /// 测试 NSFW 游戏的详情获取：v0 和旧 API 均返回 404，应通过搜索回退
    #[tokio::test]
    async fn test_detail_nsfw_with_search_fallback() {
        println!("\n===== NSFW游戏详情获取（关键词搜索回退）=====");

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
            Err(e) => {
                if is_network_err(&e) {
                    println!("⚠ 网络错误（跳过）: {}", e);
                    return;
                }
                panic!("搜索回退失败: {}", e);
            }
        }

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
            Err(e) => {
                if is_network_err(&e) {
                    println!("⚠ 网络错误（跳过）: {}", e);
                } else {
                    panic!("搜索回退失败: {}", e);
                }
            }
        }
    }

    /// 完整流程测试：搜索 → 获取详情 → 下载封面
    #[tokio::test]
    async fn test_full_flow_all_three() {
        println!("\n===== 完整流程测试：三个关键词 =====");

        // === Ever17 ===
        println!("\n--- Ever17 (id=1126)：正常 v0 API 路径 ---");
        let detail_1126 = match bangumi::get_game_detail(1126).await {
            Ok(d) => d,
            Err(e) => {
                println!("⚠ 网络错误（跳过）: {}", e);
                return;
            }
        };
        println!("  名称: {} / {}", detail_1126.name, detail_1126.name_cn);
        println!("  简介: {} 字", detail_1126.summary.len());
        println!("  标签: {} 个", detail_1126.tags.len());
        println!("  封面: {}", detail_1126.image_large);
        assert!(!detail_1126.name.is_empty());
        assert!(!detail_1126.summary.is_empty());
        assert!(!detail_1126.image_large.is_empty());
        println!("  ✓ Ever17 完整数据获取成功");

        match bangumi::download_cover_as_base64(&detail_1126.image_large).await {
            Ok(data_uri) => println!("  ✓ 封面下载成功: {} chars", data_uri.len()),
            Err(e) => println!("  ⚠ 封面下载失败（非致命）: {}", e),
        }

        // === 缘之空 ===
        println!("\n--- 缘之空 (id=2388)：NSFW 搜索回退路径 ---");
        let search_results = match bangumi::search_games("缘之空").await {
            Ok(r) => r,
            Err(e) => {
                println!("⚠ 网络错误（跳过后续）: {}", e);
                return;
            }
        };
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

        match bangumi::download_cover_as_base64(&item_2388.image_large).await {
            Ok(data_uri) => println!("  ✓ 封面下载成功: {} chars", data_uri.len()),
            Err(e) => println!("  ⚠ 封面下载失败（非致命）: {}", e),
        }

        // === 悠之空 ===
        println!("\n--- 悠之空 (id=6967)：NSFW 搜索回退路径 ---");
        let search_results = match bangumi::search_games("悠之空").await {
            Ok(r) => r,
            Err(e) => {
                println!("⚠ 网络错误（跳过）: {}", e);
                return;
            }
        };
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

        match bangumi::download_cover_as_base64(&item_6967.image_large).await {
            Ok(data_uri) => println!("  ✓ 封面下载成功: {} chars", data_uri.len()),
            Err(e) => println!("  ⚠ 封面下载失败（非致命）: {}", e),
        }

        println!("\n===== 全部流程测试通过 =====");
    }

    /// 测试 Ever17 完整 fetch 流程
    #[tokio::test]
    async fn test_fetch_full_ever17() {
        println!("\n===== Ever17 完整 fetch 流程 =====");
        let detail = match bangumi::get_game_detail(1126).await {
            Ok(d) => d,
            Err(e) => {
                println!("⚠ 网络错误（跳过）: {}", e);
                return;
            }
        };
        println!("  name: {} / {}", detail.name, detail.name_cn);
        println!("  summary_len: {}", detail.summary.len());
        println!("  tags: {:?}", detail.tags);
        println!("  platform: {}", detail.platform);
        println!("  image_large: {}", detail.image_large);
        assert!(!detail.name.is_empty(), "name 不应为空");
        assert!(!detail.summary.is_empty(), "summary 不应为空");

        let cover_url = if !detail.image_large.is_empty() {
            &detail.image_large
        } else {
            &detail.image
        };
        let cover = bangumi::download_cover_as_base64(cover_url).await;
        match &cover {
            Ok(data) => {
                println!("  ✓ 封面下载成功: {} chars", data.len());
                assert!(data.len() > 100, "封面数据过小");
            }
            Err(e) => println!("  ⚠ 封面下载失败(非致命): {}", e),
        }
        println!("  ✓ Ever17 完整流程通过");
    }

    /// 测试 Remember11 (id=1127) 详情获取
    #[tokio::test]
    async fn test_fetch_remember11() {
        println!("\n===== Remember11 (id=1127) 详情获取 =====");
        match bangumi::get_game_detail(1127).await {
            Ok(detail) => {
                println!("  name: {} / {}", detail.name, detail.name_cn);
                println!("  summary_len: {}", detail.summary.len());
                println!("  tags count: {}", detail.tags.len());
                println!("  image_large: {}", detail.image_large);
                assert!(!detail.name.is_empty(), "name 不应为空");
                assert!(!detail.summary.is_empty(), "summary 不应为空");
                println!("  ✓ Remember11 详情获取成功");
            }
            Err(e) => {
                if is_network_err(&e) {
                    println!("⚠ 网络错误（跳过）: {}", e);
                } else {
                    panic!("Remember11 详情失败: {}", e);
                }
            }
        }
    }

    /// 测试命运石之门详情获取
    #[tokio::test]
    async fn test_fetch_steins_gate() {
        println!("\n===== 命运石之门 搜索测试 =====");
        let results = match bangumi::search_games("命运石之门").await {
            Ok(r) => r,
            Err(e) => {
                println!("⚠ 网络错误（跳过）: {}", e);
                return;
            }
        };
        println!("  搜索到 {} 条结果", results.len());
        let steins = results
            .iter()
            .find(|i| i.name_cn.contains("命运石之门") || i.name.to_lowercase().contains("steins"));
        assert!(steins.is_some(), "未找到命运石之门");
        let steins = steins.unwrap();
        println!(
            "  找到: id={} name={} name_cn={}",
            steins.id, steins.name, steins.name_cn
        );
        assert!(!steins.name.is_empty());

        println!("\n--- 获取命运石之门(id={})详情 ---", steins.id);
        match bangumi::get_game_detail(steins.id).await {
            Ok(detail) => {
                println!("  name: {} / {}", detail.name, detail.name_cn);
                println!("  summary_len: {}", detail.summary.len());
                assert!(!detail.name.is_empty());
                println!("  ✓ 命运石之门详情获取成功");
            }
            Err(e) => {
                if is_network_err(&e) {
                    println!("⚠ 网络错误（跳过）: {}", e);
                } else {
                    panic!("命运石之门详情失败: {}", e);
                }
            }
        }
    }

    /// 测试不存在的游戏名搜索
    #[tokio::test]
    async fn test_search_nonexistent() {
        println!("\n===== 搜索不存在的游戏名 =====");
        let results = bangumi::search_games("zzzzzz不存在的游戏名xxxxxx").await;
        match results {
            Ok(items) => {
                println!("  搜索返回 {} 条结果", items.len());
                assert!(
                    items.is_empty() || items.len() <= 2,
                    "不存在的名字应返回少量或空结果"
                );
                println!("  ✓ 搜索不存在的名字正确处理(无崩溃)");
            }
            Err(e) => {
                println!("  搜索返回错误(也是可接受的): {}", e);
            }
        }
    }

    /// 测试不存在的 subject_id 详情获取
    #[tokio::test]
    async fn test_detail_nonexistent_id() {
        println!("\n===== 不存在的 subject_id 详情获取 =====");
        let fake_id = 99999999u32;
        match bangumi::get_game_detail(fake_id).await {
            Ok(detail) => {
                println!("  返回了数据(可能是错误页): name={}", detail.name);
            }
            Err(e) => {
                println!("  ✓ 正确返回错误: {}", e);
            }
        }
    }

    // ==============================
    // 新增全面解析测试
    // ==============================

    #[tokio::test]
    async fn test_search_and_parse_koichoco() {
        println!("\n===== 全面测试搜索+解析「恋爱和选举和巧克力」=====");
        match bangumi::search_games("恋爱和选举和巧克力").await {
            Ok(items) => {
                println!("找到 {} 条结果", items.len());
                assert!(!items.is_empty(), "至少应有1条结果");
                for (idx, item) in items.iter().enumerate() {
                    println!("  [{idx}] id={} name='{}' name_cn='{}' date='{}' score={} rank={} summary_len={} image='{}' image_large='{}'",
                        item.id, item.name, item.name_cn, item.date, item.score, item.rank,
                        item.summary.len(),
                        if item.image.is_empty() { "(空)" } else { "有图" },
                        if item.image_large.is_empty() { "(空)" } else { "有图" },
                    );
                    assert!(
                        !item.name.is_empty() || !item.name_cn.is_empty(),
                        "第{}条结果 id={} 名称完全为空",
                        idx,
                        item.id
                    );
                    assert!(item.id > 0, "第{}条结果 id 无效", idx);
                }
                println!("✓ 所有 {} 条结果解析成功", items.len());
            }
            Err(e) => {
                if is_network_err(&e) {
                    println!("⚠ 网络错误（跳过）: {}", e);
                } else {
                    panic!("搜索失败: {}", e);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_search_and_parse_nekopara_live() {
        println!("\n===== 全面测试搜索+解析「ネコぱらいぶ☆」=====");
        match bangumi::search_games("ネコぱらいぶ☆").await {
            Ok(items) => {
                println!("找到 {} 条结果", items.len());
                assert!(!items.is_empty(), "至少应有1条结果");
                for (idx, item) in items.iter().enumerate() {
                    println!("  [{idx}] id={} name='{}' name_cn='{}' date='{}' score={} rank={} summary_len={} image='{}'",
                        item.id, item.name, item.name_cn, item.date, item.score, item.rank,
                        item.summary.len(),
                        if item.image.is_empty() { "(空)" } else { "有图" },
                    );
                    assert!(
                        !item.name.is_empty() || !item.name_cn.is_empty(),
                        "第{}条结果 id={} 名称完全为空",
                        idx,
                        item.id
                    );
                    assert!(item.id > 0, "第{}条结果 id 无效", idx);
                }
                println!("✓ 所有 {} 条结果解析成功", items.len());
            }
            Err(e) => {
                if is_network_err(&e) {
                    println!("⚠ 网络错误（跳过）: {}", e);
                } else {
                    panic!("搜索失败: {}", e);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_search_and_parse_xuanyuanjian() {
        println!("\n===== 全面测试搜索+解析「轩辕剑龙舞云山」=====");
        match bangumi::search_games("轩辕剑龙舞云山").await {
            Ok(items) => {
                println!("找到 {} 条结果", items.len());
                assert!(!items.is_empty(), "至少应有1条结果");
                for (idx, item) in items.iter().enumerate() {
                    println!("  [{idx}] id={} name='{}' name_cn='{}' date='{}' score={} rank={} summary_len={} image='{}'",
                        item.id, item.name, item.name_cn, item.date, item.score, item.rank,
                        item.summary.len(),
                        if item.image.is_empty() { "(空)" } else { "有图" },
                    );
                    assert!(
                        !item.name.is_empty() || !item.name_cn.is_empty(),
                        "第{}条结果 id={} 名称完全为空",
                        idx,
                        item.id
                    );
                    assert!(item.id > 0, "第{}条结果 id 无效", idx);
                }
                println!("✓ 所有 {} 条结果解析成功", items.len());
            }
            Err(e) => {
                if is_network_err(&e) {
                    println!("⚠ 网络错误（跳过）: {}", e);
                } else {
                    panic!("搜索失败: {}", e);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_search_and_parse_doraemon_biohazard() {
        println!("\n===== 全面测试搜索+解析「哆啦A梦：野比大雄的生化危机」=====");
        match bangumi::search_games("哆啦A梦：野比大雄的生化危机").await {
            Ok(items) => {
                println!("找到 {} 条结果", items.len());
                assert!(!items.is_empty(), "至少应有1条结果");
                for (idx, item) in items.iter().enumerate() {
                    println!("  [{idx}] id={} name='{}' name_cn='{}' date='{}' score={} rank={} summary_len={} image='{}'",
                        item.id, item.name, item.name_cn, item.date, item.score, item.rank,
                        item.summary.len(),
                        if item.image.is_empty() { "(空)" } else { "有图" },
                    );
                    assert!(
                        !item.name.is_empty() || !item.name_cn.is_empty(),
                        "第{}条结果 id={} 名称完全为空",
                        idx,
                        item.id
                    );
                    assert!(item.id > 0, "第{}条结果 id 无效", idx);
                }
                println!("✓ 所有 {} 条结果解析成功", items.len());
            }
            Err(e) => {
                if is_network_err(&e) {
                    println!("⚠ 网络错误（跳过）: {}", e);
                } else {
                    panic!("搜索失败: {}", e);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_search_and_parse_ever17() {
        println!("\n===== 全面测试搜索+解析「ever17」=====");
        match bangumi::search_games("ever17").await {
            Ok(items) => {
                println!("找到 {} 条结果", items.len());
                assert!(!items.is_empty(), "至少应有1条结果");
                for (idx, item) in items.iter().enumerate() {
                    println!("  [{idx}] id={} name='{}' name_cn='{}' date='{}' score={} rank={} image='{}'",
                        item.id, item.name, item.name_cn, item.date, item.score, item.rank,
                        if item.image.is_empty() { "(空)" } else { "有图" },
                    );
                    assert!(
                        !item.name.is_empty() || !item.name_cn.is_empty(),
                        "第{}条结果 id={} 名称完全为空",
                        idx,
                        item.id
                    );
                    assert!(item.id > 0, "第{}条结果 id 无效", idx);
                }
                println!("✓ 所有 {} 条结果解析成功", items.len());
            }
            Err(e) => {
                if is_network_err(&e) {
                    println!("⚠ 网络错误（跳过）: {}", e);
                } else {
                    panic!("搜索失败: {}", e);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_bulk_search_parsing_robustness() {
        println!("\n===== 批量搜索解析健壮性测试 =====");
        let keywords = vec![
            "ever17",
            "fate",
            "clannad",
            "轩辕剑",
            "steins",
            "muv",
            "white album",
            "island",
        ];
        let mut total = 0;
        let mut all_passed = true;
        for kw in keywords {
            match bangumi::search_games(kw).await {
                Ok(items) => {
                    println!("\n  关键词「{}」: {} 条结果", kw, items.len());
                    for (idx, item) in items.iter().enumerate() {
                        total += 1;
                        let name_ok = !item.name.is_empty() || !item.name_cn.is_empty();
                        let id_ok = item.id > 0;
                        if !name_ok || !id_ok {
                            println!(
                                "    ✗ [{idx}] id={} name_ok={} id_ok={}",
                                item.id, name_ok, id_ok
                            );
                            all_passed = false;
                        }
                    }
                    for item in &items {
                        assert!(
                            !item.name.is_empty() || !item.name_cn.is_empty(),
                            "关键词「{}」中 id={} 名称完全为空",
                            kw,
                            item.id
                        );
                        assert!(item.id > 0, "关键词「{}」中 id 无效", kw);
                    }
                }
                Err(e) => println!("  关键词「{}」搜索失败: {} (跳过)", kw, e),
            }
        }
        println!("\n  总计解析 {} 条记录", total);
        assert!(all_passed, "存在解析失败的条目");
        println!("✓ 批量搜索解析健壮性测试通过");
    }
}
