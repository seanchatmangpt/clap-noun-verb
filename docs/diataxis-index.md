# Diataxis Documentation for MCP Agents: Zero to Hero Guide

**Welcome!** This is your complete guide to mastering MCP (Model Context Protocol) agents with RDF Turtle semantic CLI generation.

## Documentation Structure

This documentation follows the **Diataxis methodology**, which organizes knowledge into four complementary sections:

```
                    UNDERSTANDING
                    (Explanation)
                         |
        LEARNING          |          SOLVING
        (Tutorials) ------|------ (How-to Guides)
                         |
                    REFERENCE
```

### 🎓 **Tutorials** - Learning-Oriented
**For:** Agents new to RDF and CLI generation
**Purpose:** Build foundational knowledge through guided, hands-on learning
**Structure:** Progressive lessons from simple to complex
- Tutorial 1: Set up your first MCP agent
- Tutorial 2: Create your first RDF ontology
- Tutorial 3: Generate your first CLI
- Tutorial 4: Query ontologies with SPARQL
- Tutorial 5: Deploy production-grade CLIs

**Time commitment**: 30-60 minutes per tutorial
**Prerequisites**: Basic Rust knowledge, git configured

---

### 🛠️ **How-to Guides** - Task-Oriented
**For:** Agents solving specific problems
**Purpose:** Provide recipes and solutions for common tasks
**Structure:** Problem-solution pairs with code examples
- How to build multi-level CLIs
- How to validate ontologies
- How to optimize performance
- How to cache SPARQL results
- How to debug failing generations
- How to test generated CLIs
- How to integrate with external systems
- How to implement custom handlers

**Time commitment**: 5-15 minutes per guide
**Prerequisites**: Completed at least Tutorial 1

---

### 📚 **Reference** - Information-Oriented
**For:** Agents looking up specific details
**Purpose:** Complete, accurate technical documentation
**Structure:** Organized by topic (API, schemas, types, errors)
- API Reference (all public types and functions)
- MCP Tool Schemas (GenerateCliFromTurtle, QueryCapabilities, ExportToTurtle)
- RDF Vocabulary (cnv: namespace, all properties)
- Error Codes and troubleshooting
- Configuration options
- CLI generated code patterns

**Time commitment**: 1-5 minutes per lookup
**Prerequisites**: None - use as needed

---

### 💡 **Explanation** - Understanding-Oriented
**For:** Agents wanting to understand the "why"
**Purpose:** Build mental models and deep understanding
**Structure:** Conceptual articles with diagrams
- What is RDF and why use it?
- Type safety in CLI generation
- Performance characteristics and optimization
- MCP agent architecture
- Design patterns and best practices
- Semantic web integration

**Time commitment**: 10-30 minutes per article
**Prerequisites**: Familiarity with basic concepts

---

## Learning Paths

### 🚀 **Path 1: Complete Beginner → Productive Agent**
Estimated time: **2-3 hours**

1. **Explanation**: [What is RDF and why use it?](#explanation-rdf-basics)
2. **Tutorial**: [Set up your first MCP agent](#tutorial-1-setup)
3. **Tutorial**: [Create your first RDF ontology](#tutorial-2-first-rdf)
4. **How-to**: [Build multi-level CLIs](#howto-multi-level)
5. **Tutorial**: [Generate your first CLI](#tutorial-3-first-cli)
6. **Reference**: [API Reference](#reference-api)

### 🎯 **Path 2: Agent → Expert Developer**
Estimated time: **4-6 hours**

1. Complete Path 1
2. **Tutorial**: [Query ontologies with SPARQL](#tutorial-4-sparql)
3. **How-to**: [Optimize performance](#howto-performance)
4. **Explanation**: [Performance characteristics](#explanation-performance)
5. **How-to**: [Implement custom handlers](#howto-handlers)
6. **How-to**: [Test generated CLIs](#howto-testing)
7. **Explanation**: [Design patterns](#explanation-patterns)

### 🏭 **Path 3: Production Deployment**
Estimated time: **2-4 hours**

1. Complete Path 2
2. **Tutorial**: [Deploy production CLIs](#tutorial-5-production)
3. **How-to**: [Cache SPARQL results](#howto-caching)
4. **How-to**: [Debug failures](#howto-debugging)
5. **How-to**: [Integrate with external systems](#howto-integration)
6. **Reference**: [Error codes](#reference-errors)

---

## Quick Navigation

### By Role

**🤖 AI Agent Developer**
→ Start with [Tutorial 1](#tutorial-1-setup)
→ Focus on [Tutorials](#tutorials) and [How-to Guides](#howto-guides)

**🔧 DevOps/Infrastructure Engineer**
→ Start with [Explanation: Architecture](#explanation-architecture)
→ Focus on [How-to: Production Deployment](#tutorial-5-production)

**📚 ML Researcher**
→ Start with [Explanation: RDF and semantics](#explanation-rdf-basics)
→ Focus on [Explanation](#explanation) sections

**🎓 Student/Academic**
→ Start with [Explanation: Type Safety](#explanation-type-safety)
→ Work through all sections systematically

### By Time Available

**5 minutes**: [Reference: API Quick Lookup](#reference-api)
**15 minutes**: Any [How-to Guide](#howto-guides)
**30 minutes**: Any [Tutorial](#tutorials)
**1 hour**: [Complete Learning Path 1](#learning-paths)

---

## Getting Help

- **Can't find what you need?** → See [FAQ](#faq) or search the [Reference](#reference)
- **Something not working?** → Check [Troubleshooting](#troubleshooting) in How-to Guides
- **Want to understand the why?** → Read the [Explanation](#explanation) section
- **Need exact syntax?** → Check the [Reference](#reference) section

---

## Documentation Roadmap

### Currently Available ✅
- Tutorials 1-5 (complete)
- How-to Guides 1-8 (complete)
- API Reference (complete)
- Explanation sections 1-6 (complete)

### Coming Soon 🚧
- Interactive examples and playgrounds
- Video tutorials (30 min each)
- Advanced optimization guide
- Distributed systems with federated queries
- Custom semantic extensions

---

## Feedback and Contributions

This documentation is open-source! Have suggestions or found errors?

- **Report issues**: https://github.com/seanchatmangpt/clap-noun-verb/issues
- **Contribute improvements**: https://github.com/seanchatmangpt/clap-noun-verb/pulls
- **Join the community**: Discussions forum (coming soon)

---

**Version**: 1.0 (Updated 2026-01-06)
**Status**: Production-ready
**Last Updated**: January 6, 2026

---

## Table of Contents

### 🎓 Tutorials (Learning-Oriented)
- [Tutorial 1: Set Up Your First MCP Agent](#tutorial-1-setup)
- [Tutorial 2: Create Your First RDF Ontology](#tutorial-2-first-rdf)
- [Tutorial 3: Generate Your First CLI](#tutorial-3-first-cli)
- [Tutorial 4: Query Ontologies with SPARQL](#tutorial-4-sparql)
- [Tutorial 5: Deploy Production CLIs](#tutorial-5-production)

### 🛠️ How-to Guides (Task-Oriented)
- [How to: Build Multi-Level CLIs](#howto-multi-level)
- [How to: Validate Ontologies](#howto-validation)
- [How to: Cache SPARQL Results](#howto-caching)
- [How to: Optimize Performance](#howto-performance)
- [How to: Debug Failing Generations](#howto-debugging)
- [How to: Test Generated CLIs](#howto-testing)
- [How to: Implement Custom Handlers](#howto-handlers)
- [How to: Integrate with External Systems](#howto-integration)

### 📚 Reference (Information-Oriented)
- [API Reference: Types and Functions](#reference-api)
- [MCP Tool Schemas](#reference-mcp-tools)
- [RDF Vocabulary (cnv: namespace)](#reference-vocabulary)
- [Error Codes and Troubleshooting](#reference-errors)
- [Configuration Options](#reference-config)

### 💡 Explanation (Understanding-Oriented)
- [What is RDF and Why Use It?](#explanation-rdf-basics)
- [Type Safety in CLI Generation](#explanation-type-safety)
- [Performance Characteristics](#explanation-performance)
- [MCP Agent Architecture](#explanation-architecture)
- [Design Patterns and Best Practices](#explanation-patterns)
- [Semantic Web Integration](#explanation-semantic)

---

*Next: Start with [Tutorial 1: Set Up Your First MCP Agent](#tutorial-1-setup) →*
