# Rectangle Bin Packing Optimizer

This project was developed as part of the Optimization Algorithms course at university.

The goal of the project is to solve a variation of the 2D Rectangle Bin Packing Problem. Given a set of rectangles and a fixed box size, the task is to place all rectangles into the minimum number of boxes while ensuring that no rectangles overlap. Rectangles may also be rotated by 90 degrees to improve the packing quality.

---

## Project Goal

The objective was to implement and compare different optimization approaches discussed during the course.

The project includes:

- Greedy algorithms
- Local Search algorithms
- Random instance generation
- Solution evaluation
- Benchmarking and performance analysis
- GUI-based visualization

The implementation was written in Rust.

---

## Problem Description

Given:

- A set of rectangles with integer dimensions
- A square box with side length **L**

The algorithm must find a placement such that:

- Every rectangle is fully contained inside a box
- Rectangles do not overlap
- The total number of boxes used is minimized

Since the problem becomes computationally difficult for larger instances, heuristic optimization techniques are used.

---

## Implemented Algorithms

### Greedy Search

The Greedy approach generates an initial solution by selecting and placing rectangles according to predefined ordering strategies.

Advantages:

- Fast execution
- Good starting solutions

### Local Search

The Local Search approach improves an existing solution by exploring neighboring solutions and accepting beneficial modifications.

Implemented variants include:

- Geometry-based local search
- Rule-based local search
- Overlap-relaxation local search

These methods generally achieve better packing results than a pure Greedy approach.

---

## Features

- Generic optimization framework
- Random test instance generation
- Rectangle rotation support
- Benchmark environment
- Runtime measurement
- GUI visualization
- Comparison of different optimization strategies

---

## Project Structure

```text
src/
├── algorithm.rs
├── benchmark.rs
├── bin.rs
├── evaluator.rs
├── framework.rs
├── generator.rs
├── generic_greedy.rs
├── generic_local_search.rs
├── greedy.rs
├── gui.rs
├── instance.rs
├── local_search_geometry.rs
├── local_search_overlap.rs
└── skyline.rs
```

---

## Benchmarks

The repository contains benchmark files used to evaluate solution quality and runtime performance on larger instances.

Example metrics:

- Number of boxes used
- Runtime
- Packing efficiency
- Algorithm comparison


## What I Learned

Through this project I gained practical experience with:

- Optimization algorithms
- Local search techniques
- Heuristic problem solving
- Performance benchmarking
- Rust programming
- Software design and modular architecture

The project also helped me understand the trade-offs between solution quality and execution time when solving optimization problems.

---

## Academic Note

This repository is published for educational and portfolio purposes.

Please do not submit this work as your own academic assignment.
