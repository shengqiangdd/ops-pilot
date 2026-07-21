#!/usr/bin/env node
/**
 * Batch script v3: Simple approach — only add imports where missing.
 * Pages already manage their own loading/error state; we just need
 * to ensure the standardized components are available for manual integration.
 * 
 * Actually, let's take a different approach: wrap each page's default export
 * with a tri-state HOC.
 */
import { readFileSync, writeFileSync } from 'fs';
import { join } from 'path';

const PAGES_DIR = 'src/pages';

const PAGES = [
  { file: 'APM.tsx', skeleton: 'chart' },
  { file: 'AlertHistory.tsx', skeleton: 'list' },
  { file: 'AlertRules.tsx', skeleton: 'list' },
  { file: 'Baseline.tsx', skeleton: 'list' },
  { file: 'ChangeAnalysis.tsx', skeleton: 'list' },
  { file: 'Escalation.tsx', skeleton: 'list' },
  { file: 'Incidents.tsx', skeleton: 'list' },
  { file: 'MetricsViz.tsx', skeleton: 'chart' },
  { file: 'Monitor.tsx', skeleton: 'chart' },
  { file: 'Predictions.tsx', skeleton: 'chart' },
  { file: 'AuditLog.tsx', skeleton: 'list' },
  { file: 'Compliance.tsx', skeleton: 'list' },
  { file: 'FIM.tsx', skeleton: 'list' },
  { file: 'SecretsScan.tsx', skeleton: 'list' },
  { file: 'Security.tsx', skeleton: 'chart' },
  { file: 'Threats.tsx', skeleton: 'list' },
  { file: 'Vulnerabilities.tsx', skeleton: 'list' },
  { file: 'CMDB.tsx', skeleton: 'list' },
  { file: 'CICD.tsx', skeleton: 'list' },
  { file: 'Config.tsx', skeleton: 'detail' },
  { file: 'Hosts.tsx', skeleton: 'list' },
  { file: 'Jobs.tsx', skeleton: 'list' },
  { file: 'Reports.tsx', skeleton: 'chart' },
  { file: 'Scheduler.tsx', skeleton: 'list' },
  { file: 'Webhook.tsx', skeleton: 'list' },
  { file: 'Advisor.tsx', skeleton: 'detail' },
  { file: 'Diagnostics.tsx', skeleton: 'chart' },
  { file: 'Knowledge.tsx', skeleton: 'list' },
  { file: 'Runbook.tsx', skeleton: 'list' },
  { file: 'Timeline.tsx', skeleton: 'list' },
  { file: 'Chaos.tsx', skeleton: 'list' },
  { file: 'FinOps.tsx', skeleton: 'chart' },
  { file: 'LogIntelligence.tsx', skeleton: 'list' },
  { file: 'NotificationChannels.tsx', skeleton: 'list' },
  { file: 'OnCall.tsx', skeleton: 'detail' },
  { file: 'SLOs.tsx', skeleton: 'chart' },
  { file: 'Topology.tsx', skeleton: 'chart' },
  { file: 'SOAR.tsx', skeleton: 'list' },
  { file: 'Remediation.tsx', skeleton: 'list' },
];

let modified = 0, skipped = 0, errors = 0;

for (const page of PAGES) {
  const filePath = join(PAGES_DIR, page.file);
  try {
    let content = readFileSync(filePath, 'utf-8');
    
    // Already has LoadingState? Skip
    if (content.includes('LoadingState')) { skipped++; console.log(`SKIP: ${page.file} (already done)`); continue; }
    
    // Step 1: Add import for LoadingState, ErrorState
    if (!content.includes("from '../lib/pageStates'")) {
      let lastImportIdx = 0;
      const lines = content.split('\n');
      for (let i = 0; i < lines.length; i++) {
        if (lines[i].startsWith('import ')) lastImportIdx = i;
      }
      lines.splice(lastImportIdx + 1, 0,
        `import { LoadingState, ErrorState } from '../lib/pageStates';`
      );
      content = lines.join('\n');
    }
    
    // Step 2: Find first return statement in the component and insert guard before it
    const lines = content.split('\n');
    
    // Find the exported component
    let componentStart = -1;
    for (let i = 0; i < lines.length; i++) {
      if (/^export\s+(function|const)\s+\w+/.test(lines[i])) {
        componentStart = i;
        break;
      }
    }
    if (componentStart === -1) { skipped++; console.log(`SKIP: ${page.file} (no export found)`); continue; }
    
    // Find first return inside the component
    let returnIdx = -1;
    let depth = 0;
    for (let i = componentStart; i < lines.length; i++) {
      depth += (lines[i].match(/\{/g) || []).length;
      depth -= (lines[i].match(/\}/g) || []).length;
      if (depth === 1 && (lines[i].trim().startsWith('return (') || lines[i].trim().startsWith('return <'))) {
        returnIdx = i;
        break;
      }
    }
    if (returnIdx === -1) { skipped++; console.log(`SKIP: ${page.file} (no return found)`); continue; }
    
    // Check if there's already a loading check before this return
    const precedingLines = lines.slice(Math.max(0, returnIdx - 5), returnIdx).join('\n');
    if (precedingLines.includes('if (loading)') || precedingLines.includes('if (loading,')) {
      skipped++;
      console.log(`SKIP: ${page.file} (already has loading guard)`);
      continue;
    }
    
    // Find where to insert the guard - right before the return line
    // Also need to ensure useState is imported
    if (!content.includes('useState')) {
      for (let i = 0; i < lines.length; i++) {
        if (lines[i].startsWith("import ") && lines[i].includes("'react'")) {
          if (!lines[i].includes('useState')) {
            lines[i] = lines[i].replace("import { ", "import { useState, ");
          }
          break;
        }
      }
    }
    
    // Check if page already has loading/error state
    const text = lines.join('\n');
    const hasLoading = /const\s*\[\s*loading\s*,/.test(text);
    const hasError = /const\s*\[\s*error\s*,/.test(text);
    
    // Find insertion point for state vars (first line after component opening with a const/let)
    let stateInsertIdx = componentStart + 1;
    while (stateInsertIdx < returnIdx && lines[stateInsertIdx].trim() !== '{') {
      stateInsertIdx++;
    }
    stateInsertIdx++; // after the opening brace
    
    const insertLines = [];
    if (!hasLoading) insertLines.push(`  const [loading, setLoading] = useState(true);`);
    if (!hasError) insertLines.push(`  const [error, setError] = useState<string | null>(null);`);
    
    // Insert state vars
    if (insertLines.length > 0) {
      lines.splice(stateInsertIdx, 0, ...insertLines);
    }
    
    // Re-find return line (shifted)
    const shift = insertLines.length;
    let newReturnIdx = returnIdx + shift;
    
    // Insert guard before return
    const guard = [
      ``,
      `  if (loading) return <LoadingState skeleton="${page.skeleton}" />;`,
      `  if (error) return <ErrorState message={error} onRetry={() => window.location.reload()} />;`,
    ];
    lines.splice(newReturnIdx, 0, ...guard);
    
    writeFileSync(filePath, lines.join('\n'), 'utf-8');
    modified++;
    console.log(`OK: ${page.file}`);
  } catch (e) {
    errors++;
    console.log(`ERR: ${page.file} — ${e.message}`);
  }
}

console.log(`\nDone: ${modified} modified, ${skipped} skipped, ${errors} errors`);
