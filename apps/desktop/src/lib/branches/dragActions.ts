import { DraggableCommit, DraggableHunk, DraggableFile } from '$lib/dragging/draggables';
import { dismissToast, showError, showInfo } from '$lib/notifications/toasts';
import * as toasts from '$lib/utils/toasts';
import { filesToOwnership } from '$lib/vbranches/ownership';
import { LocalFile, type VirtualBranch } from '$lib/vbranches/types';
import type { BranchController } from '$lib/vbranches/branchController';

class BranchDragActions {
	constructor(
		private branchController: BranchController,
		private branch: VirtualBranch
	) {}

	acceptMoveCommit(data: any) {
		return data instanceof DraggableCommit && data.branchId !== this.branch.id && data.isHeadCommit;
	}

	onMoveCommit(data: DraggableCommit) {
		this.branchController.moveCommit(this.branch.id, data.commit.id);
	}

	acceptBranchDrop(data: any) {
		if (data instanceof DraggableHunk && !data.commitId && data.branchId !== this.branch.id) {
			return !data.hunk.locked;
		} else if (
			data instanceof DraggableFile &&
			data.file instanceof LocalFile &&
			this.branch.id !== data.branchId
		) {
			return !data.files.some((f) => f.locked);
		} else {
			return false;
		}
	}

	onBranchDrop(data: DraggableHunk | DraggableFile) {
		let actionName;
		if (data instanceof DraggableHunk) {
			actionName = 'hunk';
		} else if (data instanceof DraggableFile) {
			actionName = 'file';
		}
		const startedId = showInfo(`Working...`, `Move ${actionName} in-progress`);

		try {
			if (data instanceof DraggableHunk) {
				const newOwnership = `${data.hunk.filePath}:${data.hunk.id}`;
				this.branchController.updateBranchOwnership(
					this.branch.id,
					(newOwnership + '\n' + this.branch.ownership).trim()
				);
			} else if (data instanceof DraggableFile) {
				const newOwnership = filesToOwnership(data.files);
				this.branchController.updateBranchOwnership(
					this.branch.id,
					(newOwnership + '\n' + this.branch.ownership).trim()
				);
			}

			toasts.success(`Successfully moved ${actionName}`);
		} catch (e: any) {
			showError('There was a problem when moving ${actionName}', e.message);
		} finally {
			dismissToast(startedId);
		}
	}
}

export class BranchDragActionsFactory {
	constructor(private branchController: BranchController) {}

	build(branch: VirtualBranch) {
		return new BranchDragActions(this.branchController, branch);
	}
}
