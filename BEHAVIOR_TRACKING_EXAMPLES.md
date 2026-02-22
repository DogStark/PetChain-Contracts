# Behavioral Tracking System - Usage Examples

## Example 1: Recording Behavioral Issues

```rust
// Pet owner records aggressive behavior
let record_id = client.add_behavior_record(
    &pet_id,
    &BehaviorType::Aggression,
    &7,  // Severity: 7/10
    &String::from_str(&env, "Barking at strangers during walks"),
);

// Later, record improvement
client.add_behavior_record(
    &pet_id,
    &BehaviorType::Aggression,
    &4,  // Severity reduced to 4/10
    &String::from_str(&env, "Much calmer after training, only occasional barking"),
);
```

## Example 2: Training Progress Tracking

```rust
// Create training milestone
let milestone_id = client.add_training_milestone(
    &pet_id,
    &String::from_str(&env, "Basic Obedience"),
    &String::from_str(&env, "Master sit, stay, come, and heel commands"),
);

// Mark as achieved when completed
client.mark_milestone_achieved(&milestone_id);
```

## Example 3: Anxiety Management

```rust
// Initial assessment
client.add_behavior_record(
    &pet_id,
    &BehaviorType::Anxiety,
    &9,
    &String::from_str(&env, "Severe separation anxiety, destructive when alone"),
);

// After 2 weeks of training
client.add_behavior_record(
    &pet_id,
    &BehaviorType::Anxiety,
    &6,
    &String::from_str(&env, "Showing improvement, less destructive behavior"),
);

// After 1 month
client.add_behavior_record(
    &pet_id,
    &BehaviorType::Anxiety,
    &3,
    &String::from_str(&env, "Significant improvement, calm when left alone"),
);

// Retrieve improvement history
let anxiety_history = client.get_behavior_improvements(&pet_id, &BehaviorType::Anxiety);
// Returns all 3 records showing severity decrease: 9 -> 6 -> 3
```

## Example 4: Comprehensive Training Program

```rust
// Set up training milestones
let week1 = client.add_training_milestone(
    &pet_id,
    &String::from_str(&env, "Week 1: Foundation"),
    &String::from_str(&env, "Sit and stay commands"),
);

let week4 = client.add_training_milestone(
    &pet_id,
    &String::from_str(&env, "Week 4: Advanced"),
    &String::from_str(&env, "Off-leash recall and heel"),
);

let week8 = client.add_training_milestone(
    &pet_id,
    &String::from_str(&env, "Week 8: Socialization"),
    &String::from_str(&env, "Comfortable with other dogs and people"),
);

// Track progress with behavior records
client.add_behavior_record(
    &pet_id,
    &BehaviorType::Training,
    &8,
    &String::from_str(&env, "Excellent progress on basic commands"),
);

// Mark milestones as achieved
client.mark_milestone_achieved(&week1);
client.mark_milestone_achieved(&week4);

// Get all milestones to see progress
let milestones = client.get_training_milestones(&pet_id);
```

## Example 5: Veterinary Consultation

```rust
// Vet reviews complete behavioral history
let all_behaviors = client.get_behavior_history(&pet_id);

// Focus on specific concerns
let aggression_records = client.get_behavior_by_type(&pet_id, &BehaviorType::Aggression);
let anxiety_records = client.get_behavior_by_type(&pet_id, &BehaviorType::Anxiety);

// Review training progress
let milestones = client.get_training_milestones(&pet_id);

// Vet can see:
// - Behavioral trends over time
// - Severity changes
// - Training interventions
// - Timestamps for correlation with medical treatments
```

## Example 6: Socialization Tracking

```rust
// Initial socialization assessment
client.add_behavior_record(
    &pet_id,
    &BehaviorType::Socialization,
    &3,
    &String::from_str(&env, "Fearful of other dogs, avoids interaction"),
);

// Create socialization milestone
let social_milestone = client.add_training_milestone(
    &pet_id,
    &String::from_str(&env, "Dog Park Confidence"),
    &String::from_str(&env, "Comfortable playing with 3+ dogs"),
);

// Track progress
client.add_behavior_record(
    &pet_id,
    &BehaviorType::Socialization,
    &6,
    &String::from_str(&env, "Approaching other dogs, brief sniffing"),
);

client.add_behavior_record(
    &pet_id,
    &BehaviorType::Socialization,
    &9,
    &String::from_str(&env, "Playing confidently with multiple dogs"),
);

// Mark milestone achieved
client.mark_milestone_achieved(&social_milestone);
```

## Example 7: Multi-Behavior Analysis

```rust
// Record various behaviors during assessment period
client.add_behavior_record(&pet_id, &BehaviorType::Aggression, &5, 
    &String::from_str(&env, "Occasional growling at strangers"));

client.add_behavior_record(&pet_id, &BehaviorType::Anxiety, &7, 
    &String::from_str(&env, "Pacing during thunderstorms"));

client.add_behavior_record(&pet_id, &BehaviorType::Training, &8, 
    &String::from_str(&env, "Responds well to positive reinforcement"));

client.add_behavior_record(&pet_id, &BehaviorType::Socialization, &6, 
    &String::from_str(&env, "Good with familiar dogs"));

// Get complete picture
let all_records = client.get_behavior_history(&pet_id);
// Returns all 4 records for comprehensive analysis
```

## Data Analysis Patterns

### Tracking Improvement
```rust
// Get all records of a specific type
let anxiety_records = client.get_behavior_improvements(&pet_id, &BehaviorType::Anxiety);

// Analyze severity trend
for record in anxiety_records.iter() {
    println!("Date: {}, Severity: {}, Notes: {}", 
        record.recorded_at, 
        record.severity, 
        record.description
    );
}
```

### Training Effectiveness
```rust
// Get milestones
let milestones = client.get_training_milestones(&pet_id);

// Calculate completion rate
let total = milestones.len();
let achieved = milestones.iter().filter(|m| m.achieved).count();
let completion_rate = (achieved as f64 / total as f64) * 100.0;
```

## Best Practices

1. **Consistent Recording**: Log behaviors regularly for accurate trend analysis
2. **Detailed Descriptions**: Include context, triggers, and duration
3. **Severity Scale**: Use consistent criteria for severity ratings
4. **Milestone Specificity**: Make milestones measurable and achievable
5. **Progress Notes**: Update records after training sessions
6. **Correlation**: Note any medical treatments or environmental changes

## Integration with Other Systems

The behavioral tracking system integrates seamlessly with:
- **Medical Records**: Correlate behavior with health issues
- **Vaccination History**: Track behavioral changes after treatments
- **Access Control**: Share behavioral data with trainers/vets
- **Emergency Contacts**: Include behavioral notes for emergency responders
