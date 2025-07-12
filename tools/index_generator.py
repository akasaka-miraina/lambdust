#!/usr/bin/env python3
"""
Lambdust Code Index Generator

このスクリプトはLambdustコードベースを解析して、
構造体・関数・メソッドの完全なインデックスを自動生成します。

Usage:
    python tools/index_generator.py [--output docs/CODE_INDEX_GENERATED.md] [--verbose]
"""

import os
import re
import ast
import argparse
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Tuple, Optional, Set
import subprocess

class RustCodeAnalyzer:
    """Rust コードの構造を解析するクラス"""
    
    def __init__(self):
        self.struct_pattern = re.compile(r'^(?:pub\s+)?struct\s+(\w+)(?:\s*<[^>]*>)?\s*{', re.MULTILINE)
        self.enum_pattern = re.compile(r'^(?:pub\s+)?enum\s+(\w+)(?:\s*<[^>]*>)?\s*{', re.MULTILINE)
        self.fn_pattern = re.compile(r'^(?:pub\s+)?(?:async\s+)?fn\s+(\w+)(?:\s*<[^>]*>)?\s*\([^)]*\)(?:\s*->\s*[^{]+)?', re.MULTILINE)
        self.impl_pattern = re.compile(r'^impl(?:\s*<[^>]*>)?\s+(?:\w+(?:\s*<[^>]*>)?\s+for\s+)?(\w+)(?:\s*<[^>]*>)?\s*{', re.MULTILINE)
        
    def analyze_file(self, file_path: str) -> Dict:
        """単一ファイルを解析"""
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
        except Exception as e:
            print(f"Warning: Could not read {file_path}: {e}")
            return {}
            
        result = {
            'path': file_path,
            'structs': [],
            'enums': [],
            'functions': [],
            'impls': [],
            'lines': len(content.split('\n'))
        }
        
        # 構造体を検索
        for match in self.struct_pattern.finditer(content):
            line_num = content[:match.start()].count('\n') + 1
            struct_name = match.group(1)
            struct_def = self._extract_struct_definition(content, match.start())
            result['structs'].append({
                'name': struct_name,
                'line': line_num,
                'definition': struct_def,
                'visibility': 'pub' if 'pub' in match.group(0) else 'private'
            })
        
        # Enumを検索
        for match in self.enum_pattern.finditer(content):
            line_num = content[:match.start()].count('\n') + 1
            enum_name = match.group(1)
            enum_def = self._extract_enum_definition(content, match.start())
            result['enums'].append({
                'name': enum_name,
                'line': line_num,
                'definition': enum_def,
                'visibility': 'pub' if 'pub' in match.group(0) else 'private'
            })
        
        # 関数を検索
        for match in self.fn_pattern.finditer(content):
            line_num = content[:match.start()].count('\n') + 1
            fn_name = match.group(1)
            fn_signature = match.group(0).strip()
            result['functions'].append({
                'name': fn_name,
                'line': line_num,
                'signature': fn_signature,
                'visibility': 'pub' if 'pub' in fn_signature else 'private'
            })
        
        # Implブロックを検索
        for match in self.impl_pattern.finditer(content):
            line_num = content[:match.start()].count('\n') + 1
            impl_target = match.group(1)
            methods = self._extract_impl_methods(content, match.start())
            result['impls'].append({
                'target': impl_target,
                'line': line_num,
                'methods': methods
            })
        
        return result
    
    def _extract_struct_definition(self, content: str, start_pos: int) -> str:
        """構造体定義を抽出"""
        # 簡易的な実装：最初の }までを取得
        brace_count = 0
        in_struct = False
        end_pos = start_pos
        
        for i, char in enumerate(content[start_pos:], start_pos):
            if char == '{':
                brace_count += 1
                in_struct = True
            elif char == '}' and in_struct:
                brace_count -= 1
                if brace_count == 0:
                    end_pos = i + 1
                    break
        
        definition = content[start_pos:end_pos].strip()
        # 最大5行に制限
        lines = definition.split('\n')
        if len(lines) > 5:
            return '\n'.join(lines[:5]) + '\n    // ...'
        return definition
    
    def _extract_enum_definition(self, content: str, start_pos: int) -> str:
        """Enum定義を抽出"""
        return self._extract_struct_definition(content, start_pos)
    
    def _extract_impl_methods(self, content: str, start_pos: int) -> List[Dict]:
        """Implブロック内のメソッドを抽出"""
        methods = []
        # 簡易的な実装
        impl_block = self._extract_struct_definition(content, start_pos)
        
        for match in self.fn_pattern.finditer(impl_block):
            line_offset = impl_block[:match.start()].count('\n')
            global_line = content[:start_pos].count('\n') + line_offset + 1
            
            methods.append({
                'name': match.group(1),
                'line': global_line,
                'signature': match.group(0).strip(),
                'visibility': 'pub' if 'pub' in match.group(0) else 'private'
            })
        
        return methods

class IndexGenerator:
    """インデックス生成メインクラス"""
    
    def __init__(self, root_dir: str):
        self.root_dir = Path(root_dir)
        self.analyzer = RustCodeAnalyzer()
        self.analysis_results = {}
        
    def find_rust_files(self) -> List[str]:
        """Rustファイルを再帰的に検索"""
        rust_files = []
        
        for root, dirs, files in os.walk(self.root_dir):
            # target ディレクトリをスキップ
            if 'target' in dirs:
                dirs.remove('target')
            
            for file in files:
                if file.endswith('.rs'):
                    full_path = os.path.join(root, file)
                    rel_path = os.path.relpath(full_path, self.root_dir)
                    rust_files.append(rel_path)
        
        return sorted(rust_files)
    
    def analyze_codebase(self, verbose: bool = False) -> None:
        """コードベース全体を解析"""
        rust_files = self.find_rust_files()
        
        if verbose:
            print(f"Found {len(rust_files)} Rust files")
        
        for file_path in rust_files:
            if verbose:
                print(f"Analyzing: {file_path}")
            
            full_path = self.root_dir / file_path
            analysis = self.analyzer.analyze_file(str(full_path))
            
            if analysis:
                self.analysis_results[file_path] = analysis
    
    def generate_index(self, output_path: str = None) -> str:
        """インデックスを生成"""
        if not output_path:
            output_path = self.root_dir / "docs" / "CODE_INDEX_GENERATED.md"
        
        # 統計情報を計算
        stats = self._calculate_stats()
        
        # マークダウンを生成
        content = self._generate_markdown(stats)
        
        # ファイルに出力
        os.makedirs(os.path.dirname(output_path), exist_ok=True)
        with open(output_path, 'w', encoding='utf-8') as f:
            f.write(content)
        
        return content
    
    def _calculate_stats(self) -> Dict:
        """統計情報を計算"""
        stats = {
            'total_files': len(self.analysis_results),
            'total_structs': 0,
            'total_enums': 0,
            'total_functions': 0,
            'total_methods': 0,
            'total_lines': 0,
            'pub_structs': 0,
            'pub_enums': 0,
            'pub_functions': 0
        }
        
        for file_data in self.analysis_results.values():
            stats['total_structs'] += len(file_data.get('structs', []))
            stats['total_enums'] += len(file_data.get('enums', []))
            stats['total_functions'] += len(file_data.get('functions', []))
            stats['total_lines'] += file_data.get('lines', 0)
            
            # Public visibility カウント
            for struct in file_data.get('structs', []):
                if struct.get('visibility') == 'pub':
                    stats['pub_structs'] += 1
            
            for enum in file_data.get('enums', []):
                if enum.get('visibility') == 'pub':
                    stats['pub_enums'] += 1
            
            for func in file_data.get('functions', []):
                if func.get('visibility') == 'pub':
                    stats['pub_functions'] += 1
            
            # メソッド数をカウント
            for impl_data in file_data.get('impls', []):
                stats['total_methods'] += len(impl_data.get('methods', []))
        
        return stats
    
    def _generate_markdown(self, stats: Dict) -> str:
        """マークダウン形式のインデックスを生成"""
        now = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        
        content = f"""# Lambdust Code Index (Auto-Generated)

**Generated**: {now}  
**Total Files**: {stats['total_files']}  
**Total Structs**: {stats['total_structs']} (Public: {stats['pub_structs']})  
**Total Enums**: {stats['total_enums']} (Public: {stats['pub_enums']})  
**Total Functions**: {stats['total_functions']} (Public: {stats['pub_functions']})  
**Total Methods**: {stats['total_methods']}  
**Total Lines**: {stats['total_lines']:,}  

## 📋 File Index

"""
        
        # ファイル別に整理
        categories = self._categorize_files()
        
        for category, files in categories.items():
            content += f"### {category}\n\n"
            
            for file_path in sorted(files):
                file_data = self.analysis_results[file_path]
                struct_count = len(file_data.get('structs', []))
                enum_count = len(file_data.get('enums', []))
                func_count = len(file_data.get('functions', []))
                
                content += f"#### {file_path}\n"
                content += f"**Lines**: {file_data.get('lines', 0)} | "
                content += f"**Structs**: {struct_count} | "
                content += f"**Enums**: {enum_count} | "
                content += f"**Functions**: {func_count}\n\n"
                
                # Public structures
                pub_structs = [s for s in file_data.get('structs', []) if s.get('visibility') == 'pub']
                if pub_structs:
                    content += "**Public Structs**:\n"
                    for struct in pub_structs[:3]:  # 最大3個まで表示
                        content += f"- `{struct['name']}` (Line {struct['line']})\n"
                    if len(pub_structs) > 3:
                        content += f"- ... and {len(pub_structs) - 3} more\n"
                    content += "\n"
                
                # Public enums
                pub_enums = [e for e in file_data.get('enums', []) if e.get('visibility') == 'pub']
                if pub_enums:
                    content += "**Public Enums**:\n"
                    for enum in pub_enums[:3]:
                        content += f"- `{enum['name']}` (Line {enum['line']})\n"
                    if len(pub_enums) > 3:
                        content += f"- ... and {len(pub_enums) - 3} more\n"
                    content += "\n"
                
                # Key public functions
                pub_functions = [f for f in file_data.get('functions', []) if f.get('visibility') == 'pub']
                if pub_functions:
                    content += "**Key Public Functions**:\n"
                    for func in pub_functions[:5]:  # 最大5個まで表示
                        content += f"- `{func['name']}()` (Line {func['line']})\n"
                    if len(pub_functions) > 5:
                        content += f"- ... and {len(pub_functions) - 5} more\n"
                    content += "\n"
                
                content += "---\n\n"
        
        # 詳細なAPI索引
        content += self._generate_api_index()
        
        return content
    
    def _categorize_files(self) -> Dict[str, List[str]]:
        """ファイルをカテゴリ別に分類"""
        categories = {
            "🏗️ Core Infrastructure": [],
            "🚀 Evaluator System": [],
            "🎯 AST and Parsing": [],
            "🧮 Memory Management": [],
            "📝 Macro System": [],
            "🔧 Built-in Functions": [],
            "🧪 Testing": [],
            "🔌 Host Integration": [],
            "📊 Type System": [],
            "🎨 SRFI Implementation": [],
            "🛠️ Utilities": []
        }
        
        for file_path in self.analysis_results.keys():
            if file_path.startswith('src/evaluator/'):
                categories["🚀 Evaluator System"].append(file_path)
            elif file_path.startswith('src/macros/'):
                categories["📝 Macro System"].append(file_path)
            elif file_path.startswith('src/builtins/'):
                categories["🔧 Built-in Functions"].append(file_path)
            elif file_path.startswith('src/type_system/'):
                categories["📊 Type System"].append(file_path)
            elif file_path.startswith('src/srfi/'):
                categories["🎨 SRFI Implementation"].append(file_path)
            elif file_path.startswith('tests/'):
                categories["🧪 Testing"].append(file_path)
            elif 'memory' in file_path or 'pool' in file_path:
                categories["🧮 Memory Management"].append(file_path)
            elif any(core in file_path for core in ['error.rs', 'value/', 'environment.rs', 'lib.rs']):
                categories["🏗️ Core Infrastructure"].append(file_path)
            elif any(parse in file_path for parse in ['ast.rs', 'parser.rs', 'lexer.rs']):
                categories["🎯 AST and Parsing"].append(file_path)
            elif 'host' in file_path:
                categories["🔌 Host Integration"].append(file_path)
            else:
                categories["🛠️ Utilities"].append(file_path)
        
        # 空のカテゴリを削除
        return {k: v for k, v in categories.items() if v}
    
    def _generate_api_index(self) -> str:
        """詳細なAPI索引を生成"""
        content = "## 🔍 Detailed API Index\n\n"
        
        # 重要な構造体のみピックアップ
        important_structs = {}
        important_enums = {}
        
        for file_path, file_data in self.analysis_results.items():
            # 重要な構造体を特定
            for struct in file_data.get('structs', []):
                if struct.get('visibility') == 'pub' and any(important in struct['name'].lower() 
                    for important in ['error', 'value', 'environment', 'evaluator', 'context', 'parser']):
                    important_structs[struct['name']] = {
                        'file': file_path,
                        'line': struct['line'],
                        'definition': struct.get('definition', '')
                    }
            
            # 重要なEnumを特定
            for enum in file_data.get('enums', []):
                if enum.get('visibility') == 'pub' and any(important in enum['name'].lower()
                    for important in ['error', 'value', 'expr', 'token']):
                    important_enums[enum['name']] = {
                        'file': file_path,
                        'line': enum['line'],
                        'definition': enum.get('definition', '')
                    }
        
        # 重要な構造体を出力
        if important_structs:
            content += "### Key Structures\n\n"
            for name, info in sorted(important_structs.items()):
                content += f"#### {name}\n"
                content += f"**Location**: `{info['file']}:{info['line']}`\n\n"
                if info['definition']:
                    content += f"```rust\n{info['definition']}\n```\n\n"
        
        # 重要なEnumを出力
        if important_enums:
            content += "### Key Enumerations\n\n"
            for name, info in sorted(important_enums.items()):
                content += f"#### {name}\n"
                content += f"**Location**: `{info['file']}:{info['line']}`\n\n"
                if info['definition']:
                    content += f"```rust\n{info['definition']}\n```\n\n"
        
        return content

def main():
    parser = argparse.ArgumentParser(description='Generate Lambdust code index')
    parser.add_argument('--output', default='docs/CODE_INDEX_GENERATED.md',
                        help='Output file path')
    parser.add_argument('--verbose', action='store_true',
                        help='Verbose output')
    parser.add_argument('--root', default='.',
                        help='Root directory of the project')
    
    args = parser.parse_args()
    
    # プロジェクトルートの確認
    root_path = Path(args.root).absolute()
    if not (root_path / 'Cargo.toml').exists():
        print(f"Error: No Cargo.toml found in {root_path}")
        print("Please run from the project root or specify --root")
        return 1
    
    print(f"Generating index for: {root_path}")
    
    # インデックス生成
    generator = IndexGenerator(str(root_path))
    generator.analyze_codebase(verbose=args.verbose)
    
    output_path = root_path / args.output
    content = generator.generate_index(str(output_path))
    
    print(f"✅ Index generated: {output_path}")
    print(f"📊 Analyzed {len(generator.analysis_results)} files")
    
    return 0

if __name__ == '__main__':
    exit(main())