# SIGMOS Industry Examples

This directory contains comprehensive examples showcasing SIGMOS across diverse industries and use cases. Each example demonstrates the power of SIGMOS as a next-generation DSL for AI-native, composable, reactive systems.

## 🌟 Overview

SIGMOS enables declarative specification of complex, AI-driven systems with:
- **Declarative-first approach**: Define what you want, not how to implement it
- **AI-native integration**: Built-in support for LLM inference and AI services
- **Type safety**: Strong typing with validation and constraints
- **Event-driven reactivity**: Responsive to real-time data and events
- **Composable architecture**: Modular, reusable specifications
- **Extensible plugins**: MCP and REST integrations for external services

## 📁 Industry Examples

### 💰 Fintech
**[`fintech/trading-system.sigmos`](fintech/trading-system.sigmos)**
- High-frequency trading system with AI-powered risk management
- Real-time compliance checking and regulatory adherence
- Automated position management and portfolio rebalancing
- Multi-region compliance support (US, EU, APAC)
- Features: Risk assessment, market data processing, automated trading decisions

### 🏥 Healthcare
**[`healthcare/patient-monitoring.sigmos`](healthcare/patient-monitoring.sigmos)**
- Real-time patient vital signs monitoring with AI anomaly detection
- Automated alert systems for critical conditions
- Integration with hospital systems and emergency protocols
- HIPAA-compliant data handling and secure communications
- Features: Predictive health analytics, emergency response, medication tracking

### 🛒 E-commerce
**[`ecommerce/recommendation-engine.sigmos`](ecommerce/recommendation-engine.sigmos)**
- AI-powered product recommendation system with real-time personalization
- Customer segmentation and behavioral analysis
- Cross-sell and upsell optimization
- Multi-context recommendations (homepage, product page, cart, email)
- Features: Machine learning models, A/B testing, conversion tracking

### 🏭 Manufacturing
**[`manufacturing/iot-monitoring.sigmos`](manufacturing/iot-monitoring.sigmos)**
- Industrial IoT monitoring for predictive maintenance
- Quality control with AI-powered defect detection
- Production optimization and resource allocation
- Safety monitoring and emergency response
- Features: Sensor data processing, predictive analytics, automated controls

### 📦 Logistics
**[`logistics/supply-chain.sigmos`](logistics/supply-chain.sigmos)**
- AI-driven supply chain optimization and demand forecasting
- Real-time inventory management and automated replenishment
- Route optimization and fleet management
- Supplier risk assessment and diversification
- Features: Demand prediction, inventory optimization, delivery tracking

### 🔒 Cybersecurity
**[`cybersecurity/threat-detection.sigmos`](cybersecurity/threat-detection.sigmos)**
- AI-powered threat detection and automated incident response
- Real-time network monitoring and anomaly detection
- Threat intelligence integration and hunting
- Compliance reporting and audit automation
- Features: Behavioral analysis, malware detection, forensic data collection

### 🏙️ Smart Cities
**[`smart-city/urban-management.sigmos`](smart-city/urban-management.sigmos)**
- Comprehensive smart city management with AI optimization
- Traffic management and environmental monitoring
- Citizen services and emergency response coordination
- Infrastructure maintenance and resource optimization
- Features: IoT integration, predictive maintenance, citizen engagement

## 🚀 Key SIGMOS Features Demonstrated

### AI Integration
- **MCP (Model Context Protocol)**: Seamless integration with AI models and services
- **Real-time inference**: Live AI decision-making and predictions
- **Multi-model orchestration**: Combining different AI models for complex scenarios

### Event-Driven Architecture
- **Reactive systems**: Responding to real-time events and data streams
- **Complex event processing**: Handling multiple concurrent events
- **Automated workflows**: Triggering actions based on conditions and events

### Type Safety & Validation
- **Strong typing**: Comprehensive type system with validation
- **Constraint enforcement**: Business rules and data integrity
- **Schema validation**: Structured data validation and transformation

### External Integrations
- **REST APIs**: HTTP-based service integrations
- **Real-time data**: Streaming data processing and analysis
- **Third-party services**: Integration with external platforms and services

## 📊 Complexity Levels

### Beginner Examples
- [`agent.sigmos`](agent.sigmos) - Simple AI agent with basic prompt capabilities
- [`workflow.sigmos`](workflow.sigmos) - Basic workflow automation

### Intermediate Examples
- [`user-management.sigmos`](user-management.sigmos) - User lifecycle management
- [`ai-content-pipeline.sigmos`](ai-content-pipeline.sigmos) - Content processing pipeline

### Advanced Examples
- **Fintech Trading System** - Complex financial algorithms with compliance
- **Healthcare Monitoring** - Mission-critical patient care systems
- **Smart City Management** - Large-scale urban infrastructure coordination

## 🛠️ Getting Started

### Prerequisites
- SIGMOS CLI installed (`cargo install sigmos-cli`)
- Rust toolchain (1.70+)
- Access to required API services (varies by example)

### Running Examples

1. **Parse and validate a specification:**
   ```bash
   sigmos parse examples/fintech/trading-system.sigmos
   ```

2. **Execute a specification:**
   ```bash
   sigmos run examples/healthcare/patient-monitoring.sigmos
   ```

3. **Transpile to different formats:**
   ```bash
   sigmos transpile examples/ecommerce/recommendation-engine.sigmos --format json
   sigmos transpile examples/manufacturing/iot-monitoring.sigmos --format yaml
   ```

### Configuration Requirements

Each example may require specific configuration:

- **API Keys**: For external service integrations
- **Database Connections**: For data persistence
- **Network Access**: For real-time data feeds
- **Permissions**: For system integrations

Refer to individual example files for specific requirements.

## 🏗️ Architecture Patterns

### Common Patterns Demonstrated

1. **Event-Driven Processing**
   - Real-time event handling
   - Complex event correlation
   - Automated response workflows

2. **AI-First Design**
   - ML model integration
   - Predictive analytics
   - Intelligent decision-making

3. **Microservices Integration**
   - Service orchestration
   - API composition
   - Distributed system coordination

4. **Data Pipeline Architecture**
   - Stream processing
   - Data transformation
   - Real-time analytics

## 🔧 Customization Guide

### Adapting Examples

1. **Modify Input Parameters**: Adjust inputs to match your specific requirements
2. **Update Constraints**: Modify validation rules and business constraints
3. **Extend Events**: Add new event handlers for additional scenarios
4. **Integrate Services**: Replace placeholder APIs with your actual services

### Best Practices

- **Start Simple**: Begin with basic examples and gradually add complexity
- **Test Incrementally**: Validate each component before integration
- **Monitor Performance**: Use built-in metrics and logging
- **Security First**: Implement proper authentication and authorization

## 📚 Learning Path

### Recommended Order

1. **Basic Concepts**: Start with `agent.sigmos` and `workflow.sigmos`
2. **Data Processing**: Explore `ai-content-pipeline.sigmos`
3. **Business Logic**: Study `user-management.sigmos`
4. **Industry Applications**: Choose examples relevant to your domain
5. **Advanced Integration**: Combine patterns from multiple examples

### Key Concepts to Master

- **Specification Structure**: inputs, computed fields, events, constraints
- **Type System**: primitive types, objects, arrays, enums
- **Event Handling**: reactive programming patterns
- **AI Integration**: MCP protocol and model orchestration
- **External Services**: REST API integration and data exchange

## 🤝 Contributing

We welcome contributions of new industry examples! Please:

1. Follow the established patterns and conventions
2. Include comprehensive documentation and comments
3. Provide realistic use cases and scenarios
4. Test thoroughly with various inputs and conditions
5. Submit pull requests with detailed descriptions

## 📄 License

These examples are provided under the same license as the SIGMOS project. See the main project LICENSE file for details.

## 🆘 Support

- **Documentation**: [SIGMOS User Guide](../docs/user-guide.md)
- **Issues**: [GitHub Issues](https://github.com/sigmos/sigmos/issues)
- **Community**: [Discord Server](https://discord.gg/sigmos)
- **Examples**: This directory contains comprehensive real-world examples

---

*These examples demonstrate the power and flexibility of SIGMOS across diverse industries. Each specification showcases best practices for building AI-native, reactive systems that are both powerful and maintainable.*
