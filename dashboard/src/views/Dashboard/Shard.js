import React, { Component } from 'react';
import {    
    Card,
    CardBody,    
    CardHeader,    
    Col,    
    Progress,
    Row    
  } from 'reactstrap';


class Shard extends Component {
    constructor(props) {
		super(props);		
    }
    
	onUpdate = () => {
		this.setState({});
	}

	componentDidMount() {
		this.props.shard.on('change', this.onUpdate);
	}

	componentWillUnmount() {
		this.props.shard.off('change', this.onUpdate);
    }
    render() {
		return (
            <div className="progress-group">
                <div className="progress-group-header">
                <i className="icon-globe progress-group-icon"></i>
                <span className="title">{this.props.shard.name}</span>
                <span className="ml-auto font-weight-bold">{this.props.shard.current} <span className="text-muted small">({(this.props.shard.current/this.props.shard.capacity)*100}%)</span></span>  
                </div>
                <div className="progress-group-bars">
                <Progress className="progress-xs" color="success" value={(this.props.shard.current/this.props.shard.capacity)*100} />
                </div>
            </div>
        )
    }
}

export default Shard;
