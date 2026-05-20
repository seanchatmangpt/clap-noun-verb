#!/usr/bin/env node

/**
 * Robust Local unRDF Generator
 * 
 * Usage: node bin/generate.js
 * 
 * This script bypasses fragile global binaries by using local dependencies.
 * It implements the unRDF sync contract: RDF + SPARQL + Nunjucks = Code.
 */

import fs from 'fs';
import path from 'path';
import nunjucks from 'nunjucks';
import n3 from 'n3';
import toml from '@iarna/toml';

const { Parser, Store } = n3;

async function run() {
    console.log('🚀 Starting robust local unRDF generation...');

    // 1. Load config
    const configPath = path.resolve('unrdf.toml');
    if (!fs.existsSync(configPath)) {
        console.error('❌ Error: unrdf.toml not found');
        process.exit(1);
    }
    const config = toml.parse(fs.readFileSync(configPath, 'utf8'));

    // 2. Load and parse ontology
    const ontologyPath = path.resolve(config.ontology.source);
    if (!fs.existsSync(ontologyPath)) {
        console.error(`❌ Error: Ontology source ${ontologyPath} not found`);
        process.exit(1);
    }
    const ttlContent = fs.readFileSync(ontologyPath, 'utf8');
    
    const store = new Store();
    const parser = new Parser();
    
    await new Promise((resolve, reject) => {
        parser.parse(ttlContent, (error, triple, prefixes) => {
            if (error) reject(error);
            if (triple) store.addQuad(triple);
            else resolve();
        });
    });

    console.log(`✅ Loaded ontology with ${store.size} triples.`);

    // 3. Setup Nunjucks
    const env = nunjucks.configure('templates/unrdf', { autoescape: false });
    env.addFilter('zodType', (val) => {
        if (val.endsWith('integer')) return 'i32';
        if (val.endsWith('string')) return 'String';
        return 'String';
    });
    env.addFilter('replace', (str, find, replace) => str.replace(find, replace));
    env.addFilter('groupBy', (arr, key) => {
        return arr.reduce((acc, obj) => {
            const k = obj[key];
            (acc[k] = acc[k] || []).push(obj);
            return acc;
        }, {});
    });

    // 4. Execute rules
    const outputDir = config.generation.output_dir;
    if (!fs.existsSync(outputDir)) {
        fs.mkdirSync(outputDir, { recursive: true });
    }

    for (const rule of config.generation.rules) {
        console.log(`🛠️ Processing rule: ${rule.name}...`);
        
        // Simple RDF Extraction (Mocking SPARQL for robustness in shim)
        // In a full implementation, we'd use a SPARQL engine on the 'store'.
        // For this shim, we extract the structure needed by our templates.
        
        const sparql_results = [];
        
        // Extract Verbs
        const verbs = store.getQuads(null, 'http://www.w3.org/1999/02/22-rdf-syntax-ns#type', 'http://clap-noun-verb.io/ontology#Verb');
        
        for (const verbQuad of verbs) {
            const verbUri = verbQuad.subject;
            const verbName = store.getQuads(verbUri, 'http://clap-noun-verb.io/ontology#name', null)[0]?.object.value;
            const verbAbout = store.getQuads(verbUri, 'http://clap-noun-verb.io/ontology#about', null)[0]?.object.value;
            
            // Extract Arguments
            const argList = store.getQuads(verbUri, 'http://clap-noun-verb.io/ontology#arguments', null)[0]?.object;
            if (argList) {
                // Follow the RDF list
                let current = argList;
                while (current.value !== 'http://www.w3.org/1999/02/22-rdf-syntax-ns#nil') {
                    const argUri = store.getQuads(current, 'http://www.w3.org/1999/02/22-rdf-syntax-ns#first', null)[0]?.object;
                    const argName = store.getQuads(argUri, 'http://clap-noun-verb.io/ontology#name', null)[0]?.object.value;
                    const argAbout = store.getQuads(argUri, 'http://clap-noun-verb.io/ontology#about', null)[0]?.object.value;
                    const argType = store.getQuads(argUri, 'http://clap-noun-verb.io/ontology#valueType', null)[0]?.object.value;
                    const argShort = store.getQuads(argUri, 'http://clap-noun-verb.io/ontology#shortName', null)[0]?.object.value;
                    const argRequired = store.getQuads(argUri, 'http://clap-noun-verb.io/ontology#required', null)[0]?.object.value;
                    
                    sparql_results.push({
                        "?verb_name": verbName,
                        "?verb_about": verbAbout,
                        "?arg_name": argName,
                        "?arg_about": argAbout,
                        "?arg_type": argType,
                        "?arg_short": argShort,
                        "?arg_required": argRequired
                    });
                    
                    current = store.getQuads(current, 'http://www.w3.org/1999/02/22-rdf-syntax-ns#rest', null)[0]?.object;
                }
            }
        }

        // Render template
        const templateContent = fs.readFileSync(rule.template, 'utf8');
        const rendered = env.renderString(templateContent, { sparql_results });
        
        const outputPath = path.join(outputDir, rule.output_file);
        fs.writeFileSync(outputPath, rendered);
        console.log(`✨ Generated: ${outputPath}`);
    }

    console.log('🏁 unRDF sync completed successfully.');
}

run().catch(err => {
    console.error('💥 Error during generation:', err);
    process.exit(1);
});
