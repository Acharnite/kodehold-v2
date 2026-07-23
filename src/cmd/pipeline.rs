use crate::cli::PipelineArgs;
use crate::pipeline::Pipeline;

pub fn run(args: PipelineArgs) -> anyhow::Result<()> {
    let filter = args.phases.as_ref().map(|s| {
        s.split(',')
            .map(|p| p.trim().to_string())
            .collect::<Vec<_>>()
    });

    let mut pipeline = Pipeline::new(args.path.as_deref())?;

    // Run phases
    pipeline.run(filter.as_deref())?;

    // Report results
    pipeline.report(args.json)?;

    Ok(())
}
