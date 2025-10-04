#!/usr/bin/env python3
"""
简单的测试脚本，验证 MCP 服务器是否正常工作。

运行方式：
    python test_server.py
"""

import asyncio
import sys


async def test_tools_import():
    """测试导入 tools 模块"""
    print("测试 1: 导入 tools 模块...")
    try:
        from loco_mcp_server.tools import LocoTools
        tools = LocoTools()
        print("✅ Tools 模块导入成功")
        print(f"   统计信息: {tools.get_statistics()}")
        return True
    except Exception as e:
        print(f"❌ 失败: {e}")
        return False


async def test_tools_functionality():
    """测试工具功能"""
    print("\n测试 2: 测试工具功能...")
    try:
        from loco_mcp_server.tools import LocoTools
        tools = LocoTools()
        
        # 测试 generate_model（使用 mock）
        result = await tools.generate_model(
            project_path=".",
            name="test_user",
            fields={"name": "string", "email": "string"},
            with_timestamps=True
        )
        
        print("✅ 工具调用成功")
        print(f"   结果: {result.get('success')}")
        print(f"   消息: {result.get('messages', [])[:3]}")  # 只显示前3条
        return True
    except Exception as e:
        print(f"❌ 失败: {e}")
        import traceback
        traceback.print_exc()
        return False


async def test_server_creation():
    """测试服务器创建"""
    print("\n测试 3: 创建 MCP 服务器...")
    try:
        from loco_mcp_server.server import LocoMCPServer
        server = LocoMCPServer()
        print("✅ 服务器创建成功")
        print(f"   服务器名称: {server.server.name}")
        return True
    except Exception as e:
        print(f"❌ 失败: {e}")
        import traceback
        traceback.print_exc()
        return False


async def test_mcp_protocol():
    """测试 MCP 协议支持"""
    print("\n测试 4: 检查 MCP 协议支持...")
    try:
        import mcp
        from mcp.server import Server
        from mcp.types import Tool, TextContent
        print("✅ MCP SDK 可用")
        print(f"   MCP 版本: {mcp.__version__ if hasattr(mcp, '__version__') else 'unknown'}")
        return True
    except Exception as e:
        print(f"❌ 失败: {e}")
        print("   请安装 MCP SDK: pip install mcp")
        return False


async def main():
    """运行所有测试"""
    print("=" * 60)
    print("Loco MCP Server 测试套件")
    print("=" * 60)
    
    results = []
    
    # 运行测试
    results.append(await test_mcp_protocol())
    results.append(await test_tools_import())
    results.append(await test_tools_functionality())
    results.append(await test_server_creation())
    
    # 汇总结果
    print("\n" + "=" * 60)
    print(f"测试结果: {sum(results)}/{len(results)} 通过")
    print("=" * 60)
    
    if all(results):
        print("\n🎉 所有测试通过！服务器可以正常使用。")
        print("\n下一步:")
        print("1. 配置 Claude Desktop:")
        print("   编辑 claude_desktop_config.json 添加此服务器")
        print("2. 或者直接运行服务器:")
        print("   python -m loco_mcp_server.server")
        return 0
    else:
        print("\n⚠️  部分测试失败，请检查错误信息。")
        return 1


if __name__ == "__main__":
    sys.exit(asyncio.run(main()))

