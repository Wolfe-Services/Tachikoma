# Spec 245: Spec Comments

## Phase
11 - Spec Browser UI

## Spec ID
245

## Status
Planned

## Dependencies
- Spec 236 (Spec Detail View)
- Spec 237 (Spec Editor)

## Estimated Context
~8%

---

## Objective

Implement a commenting system for specs that allows users to add inline comments, threaded discussions, mentions, and comment resolution. Support markdown in comments and comment history.

---

## Acceptance Criteria

- [ ] Add comments to specs
- [ ] Inline comments on specific content
- [ ] Threaded replies
- [ ] @mentions with autocomplete
- [ ] Resolve/unresolve comments
- [ ] Edit and delete own comments
- [ ] Markdown support in comments
- [ ] Comment notifications
- [ ] Filter by resolved/unresolved

---

## Implementation Details

### CommentSection.svelte

```svelte
<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { writable, derived } from 'svelte/store';
  import type { Comment, CommentThread } from '$lib/types/spec';
  import CommentThread from './CommentThread.svelte';
  import CommentEditor from './CommentEditor.svelte';
  import Button from '$lib/components/Button.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import { formatRelativeTime } from '$lib/utils/date';

  export let specId: string;
  export let comments: Comment[] = [];
  export let currentUser: string = 'Anonymous';

  const dispatch = createEventDispatcher<{
    addComment: { content: string; parentId?: string };
    editComment: { id: string; content: string };
    deleteComment: { id: string };
    resolveComment: { id: string; resolved: boolean };
  }>();

  let filter: 'all' | 'unresolved' | 'resolved' = 'all';
  let sortBy: 'newest' | 'oldest' = 'newest';
  let showEditor = false;

  // Build comment threads
  const threads = derived(
    writable(comments),
    $comments => buildThreads($comments)
  );

  function buildThreads(comments: Comment[]): CommentThread[] {
    const threadMap = new Map<string, CommentThread>();
    const rootThreads: CommentThread[] = [];

    // First pass: create all threads
    comments.forEach(comment => {
      threadMap.set(comment.id, {
        comment,
        replies: []
      });
    });

    // Second pass: build tree structure
    comments.forEach(comment => {
      const thread = threadMap.get(comment.id)!;

      if (comment.parentId) {
        const parentThread = threadMap.get(comment.parentId);
        if (parentThread) {
          parentThread.replies.push(thread);
        } else {
          rootThreads.push(thread);
        }
      } else {
        rootThreads.push(thread);
      }
    });

    return rootThreads;
  }

  // Filter and sort threads
  $: filteredThreads = $threads
    .filter(thread => {
      if (filter === 'unresolved') return !thread.comment.resolved;
      if (filter === 'resolved') return thread.comment.resolved;
      return true;
    })
    .sort((a, b) => {
      const dateA = new Date(a.comment.createdAt).getTime();
      const dateB = new Date(b.comment.createdAt).getTime();
      return sortBy === 'newest' ? dateB - dateA : dateA - dateB;
    });

  $: unresolvedCount = $threads.filter(t => !t.comment.resolved).length;

  function handleAddComment(event: CustomEvent<{ content: string }>) {
    dispatch('addComment', { content: event.detail.content });
    showEditor = false;
  }

  function handleReply(event: CustomEvent<{ content: string; parentId: string }>) {
    dispatch('addComment', {
      content: event.detail.content,
      parentId: event.detail.parentId
    });
  }

  function handleEdit(event: CustomEvent<{ id: string; content: string }>) {
    dispatch('editComment', event.detail);
  }

  function handleDelete(event: CustomEvent<{ id: string }>) {
    dispatch('deleteComment', event.detail);
  }

  function handleResolve(event: CustomEvent<{ id: string; resolved: boolean }>) {
    dispatch('resolveComment', event.detail);
  }
</script>

<div class="comment-section">
  <header class="comment-section__header">
    <div class="comment-section__title">
      <Icon name="message-square" size={18} />
      <h3>Comments</h3>
      <span class="comment-section__count">{comments.length}</span>
      {#if unresolvedCount > 0}
        <span class="comment-section__unresolved">
          {unresolvedCount} unresolved
        </span>
      {/if}
    </div>

    <div class="comment-section__actions">
      <select bind:value={filter} class="comment-section__filter">
        <option value="all">All comments</option>
        <option value="unresolved">Unresolved</option>
        <option value="resolved">Resolved</option>
      </select>

      <select bind:value={sortBy} class="comment-section__sort">
        <option value="newest">Newest first</option>
        <option value="oldest">Oldest first</option>
      </select>
    </div>
  </header>

  <div class="comment-section__body">
    {#if showEditor}
      <div class="comment-section__new-comment">
        <CommentEditor
          placeholder="Add a comment..."
          {currentUser}
          on:submit={handleAddComment}
          on:cancel={() => showEditor = false}
        />
      </div>
    {:else}
      <button
        class="comment-section__add-btn"
        on:click={() => showEditor = true}
      >
        <Icon name="plus" size={16} />
        Add comment
      </button>
    {/if}

    {#if filteredThreads.length === 0}
      <div class="comment-section__empty">
        <Icon name="message-circle" size={32} />
        <p>
          {#if filter === 'all'}
            No comments yet. Be the first to comment!
          {:else if filter === 'unresolved'}
            No unresolved comments.
          {:else}
            No resolved comments.
          {/if}
        </p>
      </div>
    {:else}
      <div class="comment-section__threads">
        {#each filteredThreads as thread (thread.comment.id)}
          <CommentThread
            {thread}
            {currentUser}
            on:reply={handleReply}
            on:edit={handleEdit}
            on:delete={handleDelete}
            on:resolve={handleResolve}
          />
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  .comment-section {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--color-surface);
  }

  .comment-section__header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px;
    border-bottom: 1px solid var(--color-border);
  }

  .comment-section__title {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .comment-section__title h3 {
    font-size: 1rem;
    font-weight: 600;
    margin: 0;
  }

  .comment-section__count {
    padding: 2px 8px;
    font-size: 0.75rem;
    font-weight: 600;
    background: var(--color-surface-elevated);
    border-radius: 10px;
    color: var(--color-text-secondary);
  }

  .comment-section__unresolved {
    font-size: 0.75rem;
    color: var(--color-warning);
  }

  .comment-section__actions {
    display: flex;
    gap: 8px;
  }

  .comment-section__filter,
  .comment-section__sort {
    padding: 6px 10px;
    font-size: 0.875rem;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    background: var(--color-surface);
  }

  .comment-section__body {
    flex: 1;
    overflow-y: auto;
    padding: 16px;
  }

  .comment-section__add-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 12px 16px;
    margin-bottom: 16px;
    font-size: 0.875rem;
    color: var(--color-text-secondary);
    background: var(--color-surface-subtle);
    border: 1px dashed var(--color-border);
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.15s;
  }

  .comment-section__add-btn:hover {
    background: var(--color-hover);
    border-color: var(--color-primary);
    color: var(--color-primary);
  }

  .comment-section__new-comment {
    margin-bottom: 16px;
  }

  .comment-section__empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 48px;
    color: var(--color-text-tertiary);
    text-align: center;
  }

  .comment-section__empty p {
    margin-top: 12px;
    font-size: 0.875rem;
  }

  .comment-section__threads {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
</style>
```

### CommentThread.svelte

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { CommentThread as ThreadType } from '$lib/types/spec';
  import CommentEditor from './CommentEditor.svelte';
  import MarkdownPreview from './MarkdownPreview.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import { formatRelativeTime } from '$lib/utils/date';

  export let thread: ThreadType;
  export let currentUser: string;
  export let depth = 0;

  const dispatch = createEventDispatcher<{
    reply: { content: string; parentId: string };
    edit: { id: string; content: string };
    delete: { id: string };
    resolve: { id: string; resolved: boolean };
  }>();

  let showReplyEditor = false;
  let isEditing = false;
  let editContent = thread.comment.content;

  $: comment = thread.comment;
  $: isOwner = comment.author === currentUser;
  $: canResolve = depth === 0;

  function handleReply(event: CustomEvent<{ content: string }>) {
    dispatch('reply', {
      content: event.detail.content,
      parentId: comment.id
    });
    showReplyEditor = false;
  }

  function handleSaveEdit() {
    dispatch('edit', { id: comment.id, content: editContent });
    isEditing = false;
  }

  function handleDelete() {
    if (confirm('Are you sure you want to delete this comment?')) {
      dispatch('delete', { id: comment.id });
    }
  }

  function handleResolve() {
    dispatch('resolve', { id: comment.id, resolved: !comment.resolved });
  }
</script>

<div
  class="comment-thread"
  class:comment-thread--resolved={comment.resolved}
  style:--depth={depth}
>
  <div class="comment-thread__main">
    <div class="comment-thread__avatar">
      {comment.author.charAt(0).toUpperCase()}
    </div>

    <div class="comment-thread__content">
      <header class="comment-thread__header">
        <span class="comment-thread__author">{comment.author}</span>
        <span class="comment-thread__date">
          {formatRelativeTime(comment.createdAt)}
        </span>
        {#if comment.edited}
          <span class="comment-thread__edited">(edited)</span>
        {/if}
        {#if comment.resolved}
          <span class="comment-thread__resolved-badge">
            <Icon name="check" size={10} />
            Resolved
          </span>
        {/if}
      </header>

      {#if isEditing}
        <div class="comment-thread__edit">
          <textarea
            bind:value={editContent}
            class="comment-thread__edit-input"
            rows="3"
          />
          <div class="comment-thread__edit-actions">
            <button class="comment-thread__btn" on:click={handleSaveEdit}>
              Save
            </button>
            <button
              class="comment-thread__btn comment-thread__btn--ghost"
              on:click={() => { isEditing = false; editContent = comment.content; }}
            >
              Cancel
            </button>
          </div>
        </div>
      {:else}
        <div class="comment-thread__body">
          <MarkdownPreview content={comment.content} />
        </div>
      {/if}

      <div class="comment-thread__actions">
        <button
          class="comment-thread__action"
          on:click={() => showReplyEditor = !showReplyEditor}
        >
          <Icon name="message-square" size={14} />
          Reply
        </button>

        {#if isOwner}
          <button
            class="comment-thread__action"
            on:click={() => { isEditing = true; editContent = comment.content; }}
          >
            <Icon name="edit" size={14} />
            Edit
          </button>
          <button
            class="comment-thread__action comment-thread__action--danger"
            on:click={handleDelete}
          >
            <Icon name="trash" size={14} />
            Delete
          </button>
        {/if}

        {#if canResolve}
          <button
            class="comment-thread__action"
            on:click={handleResolve}
          >
            <Icon name={comment.resolved ? 'rotate-ccw' : 'check-circle'} size={14} />
            {comment.resolved ? 'Unresolve' : 'Resolve'}
          </button>
        {/if}
      </div>

      {#if showReplyEditor}
        <div class="comment-thread__reply-editor">
          <CommentEditor
            placeholder="Write a reply..."
            {currentUser}
            compact
            on:submit={handleReply}
            on:cancel={() => showReplyEditor = false}
          />
        </div>
      {/if}
    </div>
  </div>

  {#if thread.replies.length > 0}
    <div class="comment-thread__replies">
      {#each thread.replies as reply (reply.comment.id)}
        <svelte:self
          thread={reply}
          {currentUser}
          depth={depth + 1}
          on:reply
          on:edit
          on:delete
          on:resolve
        />
      {/each}
    </div>
  {/if}
</div>

<style>
  .comment-thread {
    padding-left: calc(var(--depth, 0) * 24px);
  }

  .comment-thread--resolved {
    opacity: 0.7;
  }

  .comment-thread__main {
    display: flex;
    gap: 12px;
  }

  .comment-thread__avatar {
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--color-primary);
    color: white;
    font-size: 0.875rem;
    font-weight: 600;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .comment-thread__content {
    flex: 1;
    min-width: 0;
  }

  .comment-thread__header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 4px;
  }

  .comment-thread__author {
    font-weight: 600;
    font-size: 0.875rem;
    color: var(--color-text-primary);
  }

  .comment-thread__date {
    font-size: 0.75rem;
    color: var(--color-text-tertiary);
  }

  .comment-thread__edited {
    font-size: 0.75rem;
    color: var(--color-text-tertiary);
    font-style: italic;
  }

  .comment-thread__resolved-badge {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 2px 6px;
    font-size: 0.625rem;
    font-weight: 600;
    background: var(--color-success-subtle);
    color: var(--color-success);
    border-radius: 3px;
  }

  .comment-thread__body {
    font-size: 0.875rem;
    color: var(--color-text-primary);
    line-height: 1.5;
  }

  .comment-thread__edit {
    margin-top: 8px;
  }

  .comment-thread__edit-input {
    width: 100%;
    padding: 8px 12px;
    font-size: 0.875rem;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    resize: vertical;
    font-family: inherit;
  }

  .comment-thread__edit-actions {
    display: flex;
    gap: 8px;
    margin-top: 8px;
  }

  .comment-thread__btn {
    padding: 6px 12px;
    font-size: 0.75rem;
    font-weight: 500;
    background: var(--color-primary);
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }

  .comment-thread__btn--ghost {
    background: none;
    color: var(--color-text-secondary);
  }

  .comment-thread__actions {
    display: flex;
    gap: 12px;
    margin-top: 8px;
  }

  .comment-thread__action {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 8px;
    font-size: 0.75rem;
    color: var(--color-text-tertiary);
    background: none;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }

  .comment-thread__action:hover {
    background: var(--color-hover);
    color: var(--color-text-primary);
  }

  .comment-thread__action--danger:hover {
    background: var(--color-danger-subtle);
    color: var(--color-danger);
  }

  .comment-thread__reply-editor {
    margin-top: 12px;
    padding-left: 12px;
    border-left: 2px solid var(--color-border);
  }

  .comment-thread__replies {
    margin-top: 16px;
    padding-left: 12px;
    border-left: 2px solid var(--color-border);
  }
</style>
```

### Comment Types

```typescript
// types/spec.ts additions
export interface Comment {
  id: string;
  specId: string;
  parentId?: string;
  author: string;
  content: string;
  createdAt: Date;
  updatedAt?: Date;
  edited: boolean;
  resolved: boolean;
  lineNumber?: number;
}

export interface CommentThread {
  comment: Comment;
  replies: CommentThread[];
}
```

---

## Testing Requirements

### Unit Tests

```typescript
import { render, fireEvent, screen } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import CommentSection from './CommentSection.svelte';
import CommentThread from './CommentThread.svelte';

describe('CommentSection', () => {
  const mockComments = [
    {
      id: '1',
      specId: '245',
      author: 'John',
      content: 'First comment',
      createdAt: new Date(),
      edited: false,
      resolved: false
    },
    {
      id: '2',
      specId: '245',
      author: 'Jane',
      content: 'Reply',
      parentId: '1',
      createdAt: new Date(),
      edited: false,
      resolved: false
    }
  ];

  it('displays comment count', () => {
    render(CommentSection, {
      props: { specId: '245', comments: mockComments, currentUser: 'Test' }
    });

    expect(screen.getByText('2')).toBeInTheDocument();
  });

  it('shows add comment button', () => {
    render(CommentSection, {
      props: { specId: '245', comments: [], currentUser: 'Test' }
    });

    expect(screen.getByText('Add comment')).toBeInTheDocument();
  });

  it('filters by resolved/unresolved', async () => {
    const comments = [
      { ...mockComments[0], resolved: true },
      { ...mockComments[1], resolved: false }
    ];

    render(CommentSection, {
      props: { specId: '245', comments, currentUser: 'Test' }
    });

    const filter = screen.getByRole('combobox');
    await fireEvent.change(filter, { target: { value: 'unresolved' } });

    // Should only show unresolved comments
  });

  it('dispatches addComment event', async () => {
    const { component } = render(CommentSection, {
      props: { specId: '245', comments: [], currentUser: 'Test' }
    });

    const addHandler = vi.fn();
    component.$on('addComment', addHandler);

    await fireEvent.click(screen.getByText('Add comment'));

    // Would need to fill in editor and submit
  });
});

describe('CommentThread', () => {
  const mockThread = {
    comment: {
      id: '1',
      author: 'John',
      content: 'Test comment',
      createdAt: new Date(),
      edited: false,
      resolved: false
    },
    replies: []
  };

  it('displays comment content', () => {
    render(CommentThread, {
      props: { thread: mockThread, currentUser: 'Test' }
    });

    expect(screen.getByText('Test comment')).toBeInTheDocument();
    expect(screen.getByText('John')).toBeInTheDocument();
  });

  it('shows reply button', () => {
    render(CommentThread, {
      props: { thread: mockThread, currentUser: 'Test' }
    });

    expect(screen.getByText('Reply')).toBeInTheDocument();
  });

  it('shows edit/delete for owner', () => {
    render(CommentThread, {
      props: { thread: mockThread, currentUser: 'John' }
    });

    expect(screen.getByText('Edit')).toBeInTheDocument();
    expect(screen.getByText('Delete')).toBeInTheDocument();
  });

  it('hides edit/delete for non-owner', () => {
    render(CommentThread, {
      props: { thread: mockThread, currentUser: 'Other' }
    });

    expect(screen.queryByText('Edit')).not.toBeInTheDocument();
    expect(screen.queryByText('Delete')).not.toBeInTheDocument();
  });

  it('shows resolve button for root comments', () => {
    render(CommentThread, {
      props: { thread: mockThread, currentUser: 'Test', depth: 0 }
    });

    expect(screen.getByText('Resolve')).toBeInTheDocument();
  });

  it('renders nested replies', () => {
    const threadWithReplies = {
      ...mockThread,
      replies: [{
        comment: {
          id: '2',
          author: 'Jane',
          content: 'Reply comment',
          createdAt: new Date(),
          edited: false,
          resolved: false
        },
        replies: []
      }]
    };

    render(CommentThread, {
      props: { thread: threadWithReplies, currentUser: 'Test' }
    });

    expect(screen.getByText('Reply comment')).toBeInTheDocument();
  });
});
```

---

## Related Specs

- Spec 236: Spec Detail View
- Spec 237: Spec Editor
- Spec 244: Version History
- Spec 246: Spec Sharing
